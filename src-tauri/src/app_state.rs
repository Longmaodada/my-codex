use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
    },
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{
    codex::ActivityDetector,
    database::Database,
    error::{AppError, AppResult},
    notifications::NotificationController,
    quota::{
        CapabilityProbe, ProviderKind, ProviderOutput, QuotaService, QuotaSnapshot, QuotaStatus,
    },
    settings::{AppSettings, RefreshMode},
};

const BACKOFF_SECONDS: [i64; 5] = [10, 30, 60, 120, 300];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshStatus {
    pub refreshing: bool,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub next_refresh_at: DateTime<Utc>,
    pub consecutive_failures: u8,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResult {
    pub output: ProviderOutput,
    pub skipped_concurrent: bool,
    pub refresh_status: RefreshStatus,
}

#[derive(Debug)]
struct RefreshSchedule {
    last_attempt_at: Option<DateTime<Utc>>,
    last_success_at: Option<DateTime<Utc>>,
    next_refresh_at: DateTime<Utc>,
    consecutive_failures: u8,
}

pub struct AppState {
    app: AppHandle,
    pub database: Arc<Database>,
    settings: RwLock<AppSettings>,
    quota_service: QuotaService,
    latest_quota: RwLock<ProviderOutput>,
    refresh_schedule: Mutex<RefreshSchedule>,
    refreshing: AtomicBool,
    activity_detector: ActivityDetector,
    notifications: NotificationController,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| AppError::Other(error.to_string()))?;
        let database = Arc::new(Database::open(&data_dir.join("my-codex.sqlite3"))?);
        let settings = database.load_settings()?;
        let cached = database.latest_quota_snapshot()?;
        let cached_success_at = cached.as_ref().map(|snapshot| snapshot.captured_at);
        let latest_quota = if let Some(mut snapshot) = cached {
            snapshot.status = QuotaStatus::Stale;
            snapshot.message = Some("正在刷新；当前显示上次成功取得的官方额度。".into());
            ProviderOutput {
                provider: snapshot.provider,
                capabilities: CapabilityProbe::default(),
                snapshot,
            }
        } else {
            ProviderOutput {
                provider: ProviderKind::Unavailable,
                capabilities: CapabilityProbe::default(),
                snapshot: QuotaSnapshot::unavailable(
                    ProviderKind::Unavailable,
                    "尚未取得额度；不会显示推测数字。",
                ),
            }
        };
        Ok(Self {
            app: app.clone(),
            database,
            settings: RwLock::new(settings),
            quota_service: QuotaService::new(None),
            latest_quota: RwLock::new(latest_quota),
            refresh_schedule: Mutex::new(RefreshSchedule {
                last_attempt_at: None,
                last_success_at: cached_success_at,
                next_refresh_at: Utc::now(),
                consecutive_failures: 0,
            }),
            refreshing: AtomicBool::new(false),
            activity_detector: ActivityDetector::discover(),
            notifications: NotificationController::default(),
        })
    }

    pub fn settings(&self) -> AppResult<AppSettings> {
        self.settings
            .read()
            .map(|value| value.clone())
            .map_err(|_| AppError::StatePoisoned)
    }

    pub fn update_settings(&self, settings: AppSettings) -> AppResult<AppSettings> {
        let settings = settings.validate()?;
        self.database.save_settings(&settings)?;
        *self.settings.write().map_err(|_| AppError::StatePoisoned)? = settings.clone();
        Ok(settings)
    }

    pub fn current_quota(&self) -> AppResult<ProviderOutput> {
        self.latest_quota
            .read()
            .map(|value| value.clone())
            .map_err(|_| AppError::StatePoisoned)
    }

    pub fn refresh_status(&self) -> AppResult<RefreshStatus> {
        let schedule = self
            .refresh_schedule
            .lock()
            .map_err(|_| AppError::StatePoisoned)?;
        Ok(RefreshStatus {
            refreshing: self.refreshing.load(Ordering::Acquire),
            last_attempt_at: schedule.last_attempt_at,
            last_success_at: schedule.last_success_at,
            next_refresh_at: schedule.next_refresh_at,
            consecutive_failures: schedule.consecutive_failures,
        })
    }

    pub fn refresh_due(&self) -> bool {
        self.refresh_schedule
            .lock()
            .map(|schedule| Utc::now() >= schedule.next_refresh_at)
            .unwrap_or(false)
    }

    pub fn recommended_refresh_seconds(&self) -> u64 {
        if let Ok(settings) = self.settings() {
            if let RefreshMode::Fixed(seconds) = settings.refresh_mode {
                return seconds.clamp(10, 3600);
            }
        }
        let reset_soon = self
            .current_quota()
            .ok()
            .and_then(|output| output.snapshot.reset_at)
            .map(|reset_at| {
                let remaining = reset_at.signed_duration_since(Utc::now()).num_seconds();
                (0..=300).contains(&remaining)
            })
            .unwrap_or(false);
        if reset_soon
            || self
                .activity_detector
                .is_recently_active(Duration::from_secs(90))
        {
            return 10;
        }
        let windows_visible = ["floating", "dashboard"]
            .into_iter()
            .filter_map(|label| self.app.get_webview_window(label))
            .any(|window| window.is_visible().unwrap_or(false));
        if windows_visible {
            30
        } else {
            60
        }
    }

    pub async fn refresh_quota(&self, desired_interval_seconds: u64) -> AppResult<RefreshResult> {
        if self
            .refreshing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            // Two Tauri windows bootstrap at nearly the same time. Join the
            // in-flight refresh so both receive the same first snapshot while
            // still keeping exactly one app-server process/fetch active.
            let started = std::time::Instant::now();
            while self.refreshing.load(Ordering::Acquire)
                && started.elapsed() < Duration::from_secs(13)
            {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            return Ok(RefreshResult {
                output: self.current_quota()?,
                skipped_concurrent: true,
                refresh_status: self.refresh_status()?,
            });
        }
        let _guard = RefreshFlagGuard(&self.refreshing);
        let now = Utc::now();
        {
            let mut schedule = self
                .refresh_schedule
                .lock()
                .map_err(|_| AppError::StatePoisoned)?;
            schedule.last_attempt_at = Some(now);
        }
        let mock_mode = self.settings()?.mock_mode;
        let fetched = self
            .quota_service
            .fetch(mock_mode)
            .await
            .unwrap_or_else(|_| ProviderOutput {
                provider: ProviderKind::AppServer,
                capabilities: CapabilityProbe::default(),
                snapshot: QuotaSnapshot::unavailable(
                    ProviderKind::AppServer,
                    "Codex app-server 刷新失败；未读取认证文件或私有接口。",
                ),
            });
        let successful = fetched.snapshot.status == QuotaStatus::Available;
        let output = if successful {
            if fetched.provider != ProviderKind::Mock {
                self.database.insert_quota_snapshot(&fetched.snapshot)?;
                let notification_settings = self.settings()?.notifications;
                let _ = self.notifications.evaluate(
                    &self.app,
                    &notification_settings,
                    &fetched.snapshot,
                );
            }
            fetched
        } else if fetched.snapshot.status == QuotaStatus::Unavailable {
            if let Some(mut stale) = self.database.latest_quota_snapshot()? {
                stale.status = QuotaStatus::Stale;
                stale.message =
                    Some("实时刷新失败；显示上次成功数据，时间以 lastUpdated 为准。".into());
                ProviderOutput {
                    provider: fetched.provider,
                    capabilities: fetched.capabilities,
                    snapshot: stale,
                }
            } else {
                fetched
            }
        } else {
            // Signed-out and incompatible/schema-changed states must remain
            // visible and must not inherit numeric values from a prior account.
            fetched
        };
        *self
            .latest_quota
            .write()
            .map_err(|_| AppError::StatePoisoned)? = output.clone();
        {
            let mut schedule = self
                .refresh_schedule
                .lock()
                .map_err(|_| AppError::StatePoisoned)?;
            if successful {
                schedule.consecutive_failures = 0;
                schedule.last_success_at = Some(now);
                schedule.next_refresh_at = now
                    + chrono::Duration::seconds(desired_interval_seconds.clamp(10, 3600) as i64);
            } else {
                let index = usize::from(schedule.consecutive_failures)
                    .min(BACKOFF_SECONDS.len().saturating_sub(1));
                schedule.next_refresh_at = now + chrono::Duration::seconds(BACKOFF_SECONDS[index]);
                schedule.consecutive_failures = schedule.consecutive_failures.saturating_add(1);
            }
        }
        Ok(RefreshResult {
            output,
            skipped_concurrent: false,
            refresh_status: self.refresh_status()?,
        })
    }

    pub fn notify_test(&self) -> AppResult<()> {
        self.notifications.test(&self.app)
    }
}

struct RefreshFlagGuard<'a>(&'a AtomicBool);

impl Drop for RefreshFlagGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
