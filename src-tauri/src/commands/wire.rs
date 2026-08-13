use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    analytics::{
        DataSource, DashboardAnalytics, ModelRankItem, ProjectRankItem, SkillRankItem,
        TokenBreakdown as InternalTokens, TrendPoint,
    },
    app_state::RefreshStatus,
    quota::{ProviderKind, ProviderOutput, QuotaStatus, QuotaWindow},
    settings::{
        AppSettings as InternalSettings, Language, QuotaWindowPreference, RefreshMode, ThemeMode,
    },
};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricSource {
    Official,
    Local,
    Estimated,
    Mock,
}

impl MetricSource {
    fn from_internal(source: DataSource, provider: Option<ProviderKind>) -> Self {
        if provider == Some(ProviderKind::Mock) {
            Self::Mock
        } else {
            match source {
                DataSource::Official => Self::Official,
                DataSource::Local => Self::Local,
                DataSource::Estimated => Self::Estimated,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireTokenBreakdown {
    pub input: i64,
    pub output: i64,
    pub cached: i64,
    pub reasoning: i64,
    pub total: i64,
}

impl From<InternalTokens> for WireTokenBreakdown {
    fn from(value: InternalTokens) -> Self {
        Self {
            input: value.input_tokens,
            output: value.output_tokens,
            cached: value.cached_input_tokens,
            reasoning: value.reasoning_tokens,
            total: value.total_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireQuotaWindow {
    pub id: &'static str,
    pub label: String,
    pub remaining_percent: Option<f64>,
    pub used_percent: Option<f64>,
    pub reset_at: Option<DateTime<Utc>>,
    pub window_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireQuotaSnapshot {
    pub status: &'static str,
    pub plan: Option<String>,
    pub provider: &'static str,
    pub source: MetricSource,
    pub sampled_at: DateTime<Utc>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub windows: Vec<WireQuotaWindow>,
    pub message: Option<String>,
}

impl WireQuotaSnapshot {
    pub fn from_output(output: ProviderOutput, refresh: &RefreshStatus) -> Self {
        let snapshot = output.snapshot;
        let status = match snapshot.status {
            QuotaStatus::Available => "ok",
            QuotaStatus::Stale => "stale",
            QuotaStatus::SignedOut => "signed_out",
            QuotaStatus::Unavailable => "unavailable",
            QuotaStatus::Incompatible => "schema_changed",
        };
        let provider = match snapshot.provider {
            ProviderKind::AppServer => "codex-app-server",
            ProviderKind::Mock => "mock",
            ProviderKind::Unavailable => "unavailable",
        };
        let source = MetricSource::from_internal(snapshot.source, Some(snapshot.provider));
        let mut windows = Vec::with_capacity(2);
        if let Some(window) = snapshot.primary.as_ref() {
            windows.push(map_quota_window("primary", window));
        }
        if let Some(window) = snapshot.secondary.as_ref() {
            windows.push(map_quota_window("secondary", window));
        }
        Self {
            status,
            plan: snapshot.plan,
            provider,
            source,
            sampled_at: snapshot.captured_at,
            last_success_at: refresh.last_success_at,
            windows,
            message: snapshot.message,
        }
    }
}

fn map_quota_window(id: &'static str, window: &QuotaWindow) -> WireQuotaWindow {
    let label = window.window_duration_minutes.map_or_else(
        || if id == "primary" { "主要额度".into() } else { "次要额度".into() },
        |minutes| match minutes {
            0..=59 => format!("{minutes} 分钟额度"),
            60..=1439 if minutes % 60 == 0 => format!("{} 小时额度", minutes / 60),
            _ if minutes % 1440 == 0 => format!("{} 天额度", minutes / 1440),
            _ => format!("{minutes} 分钟额度"),
        },
    );
    WireQuotaWindow {
        id,
        label,
        remaining_percent: Some(window.remaining_percent),
        used_percent: Some(window.used_percent),
        reset_at: window.resets_at,
        window_minutes: window.window_duration_minutes,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireDailyUsage {
    pub date: String,
    #[serde(flatten)]
    pub tokens: WireTokenBreakdown,
    pub requests: i64,
    pub sessions: i64,
    pub active_seconds: i64,
    pub cache_hit_ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireProjectUsage {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(flatten)]
    pub tokens: WireTokenBreakdown,
    pub sessions: i64,
    pub requests: i64,
    pub active_seconds: i64,
    pub last_active_at: String,
    pub source: MetricSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireSkillUsage {
    pub name: String,
    pub invocations: i64,
    #[serde(flatten)]
    pub tokens: WireTokenBreakdown,
    pub projects: Vec<String>,
    pub last_used_at: String,
    pub confidence: f64,
    pub source: MetricSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireModelUsage {
    pub name: String,
    #[serde(flatten)]
    pub tokens: WireTokenBreakdown,
    pub requests: i64,
    pub projects: i64,
    pub cache_hit_ratio: f64,
    pub source: MetricSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WireUsageSummary {
    pub today: WireTokenBreakdown,
    pub week: WireTokenBreakdown,
    pub last7_days: WireTokenBreakdown,
    pub lifetime: WireTokenBreakdown,
    pub requests_today: i64,
    pub sessions_today: i64,
    pub tasks_today: i64,
    pub cache_hit_ratio: f64,
    pub daily: Vec<WireDailyUsage>,
    pub projects: Vec<WireProjectUsage>,
    pub skills: Vec<WireSkillUsage>,
    pub models: Vec<WireModelUsage>,
}

impl From<DashboardAnalytics> for WireUsageSummary {
    fn from(value: DashboardAnalytics) -> Self {
        let requests_today = value.today.requests;
        let sessions_today = value.today.sessions;
        Self {
            today: value.today.tokens.into(),
            week: value.seven_days.tokens.into(),
            last7_days: value.seven_days.tokens.into(),
            lifetime: value.all_time.tokens.into(),
            requests_today,
            sessions_today,
            // Codex logs do not expose a stable standalone task counter.
            tasks_today: 0,
            cache_hit_ratio: value.cache.cache_hit_ratio,
            daily: fill_last_ninety_days(value.cache.trend),
            projects: value.projects.into_iter().map(map_project).collect(),
            skills: value.skills.into_iter().map(map_skill).collect(),
            models: value.models.into_iter().map(map_model).collect(),
        }
    }
}

impl WireUsageSummary {
    pub fn mock() -> Self {
        let split = |total: i64| WireTokenBreakdown {
            input: total.saturating_mul(51) / 100,
            output: total.saturating_mul(21) / 100,
            cached: total.saturating_mul(23) / 100,
            reasoning: total.saturating_mul(5) / 100,
            total,
        };
        // A coherent 90-day series: older 83 days sum to 1.8B, the latest
        // seven days sum to 6.8B, and today is 2.94B. Therefore the chart,
        // 7-day card, and 8.6B lifetime card never contradict one another.
        let older_base = 1_800_000_000_i64 / 83;
        let older_remainder = 1_800_000_000_i64 % 83;
        let recent = [410_000_000_i64, 520_000_000, 610_000_000, 690_000_000,
            760_000_000, 870_000_000, 2_940_000_000];
        let daily = (0_i64..90)
            .rev()
            .map(|offset| {
                let date = (chrono::Local::now().date_naive()
                    - chrono::Duration::days(offset))
                .to_string();
                let day_index = 89 - offset;
                let total = if day_index < 83 {
                    older_base + i64::from(day_index < older_remainder)
                } else {
                    recent[(day_index - 83) as usize]
                };
                WireDailyUsage {
                    date,
                    tokens: split(total),
                    requests: 12 + day_index % 173,
                    sessions: 1 + day_index % 22,
                    active_seconds: 900 + (day_index % 15).saturating_mul(840),
                    cache_hit_ratio: (58 + day_index % 27) as f64,
                }
            })
            .collect();
        let project = |id: &str,
                       name: &str,
                       path: &str,
                       total: i64,
                       sessions: i64,
                       requests: i64,
                       active_seconds: i64| WireProjectUsage {
            id: id.into(),
            name: name.into(),
            path: path.into(),
            tokens: split(total),
            sessions,
            requests,
            active_seconds,
            last_active_at: Utc::now().to_rfc3339(),
            source: MetricSource::Mock,
        };
        let skill = |name: &str, invocations: i64, total: i64, project: &str| WireSkillUsage {
            name: name.into(),
            invocations,
            tokens: split(total),
            projects: vec![project.into()],
            last_used_at: Utc::now().to_rfc3339(),
            confidence: 1.0,
            source: MetricSource::Mock,
        };
        let model = |name: &str, total: i64, requests: i64, projects: i64, cache: f64| {
            WireModelUsage {
                name: name.into(),
                tokens: split(total),
                requests,
                projects,
                cache_hit_ratio: cache,
                source: MetricSource::Mock,
            }
        };
        Self {
            today: split(2_940_000_000),
            week: split(2_940_000_000),
            last7_days: split(6_800_000_000),
            lifetime: split(8_600_000_000),
            requests_today: 184,
            sessions_today: 22,
            tasks_today: 37,
            cache_hit_ratio: 80.0,
            daily,
            projects: vec![
                project("mock-project-config", "Code项目配置生成工具", "D:\\Projects\\code-project-config", 2_580_000_000, 49, 184, 20_484),
                project("mock-skills-2", "hatch-pet-users-zhitong-codex-skills-2", "D:\\Projects\\hatch-pet-users-zhitong-codex-skills-2", 2_400_000_000, 53, 167, 15_840),
                project("mock-skills", "hatch-pet-users-zhitong-codex-skills", "D:\\Projects\\hatch-pet-users-zhitong-codex-skills", 563_200_000, 34, 96, 11_460),
            ],
            skills: vec![
                skill("frontend-design", 42, 1_840_000_000, "Code项目配置生成工具"),
                skill("github", 31, 1_220_000_000, "hatch-pet-users-zhitong-codex-skills-2"),
                skill("browser", 24, 860_000_000, "Code项目配置生成工具"),
                skill("security-analysis", 18, 620_000_000, "内网代理池维护"),
            ],
            models: vec![
                model("GPT-5.6", 3_180_000_000, 86, 5, 82.0),
                model("GPT-5.6 Sol", 2_720_000_000, 54, 3, 80.0),
                model("GPT-5.5", 1_410_000_000, 31, 4, 74.0),
            ],
        }
    }
}

fn map_daily(value: TrendPoint) -> WireDailyUsage {
    WireDailyUsage {
        date: value.bucket,
        tokens: value.tokens.into(),
        requests: value.requests,
        sessions: value.sessions,
        active_seconds: value.active_seconds,
        cache_hit_ratio: value.cache_hit_ratio,
    }
}

fn fill_last_ninety_days(values: Vec<TrendPoint>) -> Vec<WireDailyUsage> {
    let mut values: HashMap<String, WireDailyUsage> = values
        .into_iter()
        .map(|value| (value.bucket.clone(), map_daily(value)))
        .collect();
    (0_i64..90)
        .rev()
        .map(|offset| {
            let date = (chrono::Local::now().date_naive()
                - chrono::Duration::days(offset))
            .to_string();
            values.remove(&date).unwrap_or_else(|| WireDailyUsage {
                date,
                tokens: InternalTokens::default().into(),
                requests: 0,
                sessions: 0,
                active_seconds: 0,
                cache_hit_ratio: 0.0,
            })
        })
        .collect()
}

fn map_project(value: ProjectRankItem) -> WireProjectUsage {
    WireProjectUsage {
        id: value.project_id,
        name: value.project_name,
        path: value.project_path.unwrap_or_default(),
        tokens: value.tokens.into(),
        sessions: value.sessions,
        requests: value.requests,
        active_seconds: value.active_seconds,
        last_active_at: value
            .last_used_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
        source: MetricSource::from_internal(value.source, None),
    }
}

fn map_skill(value: SkillRankItem) -> WireSkillUsage {
    WireSkillUsage {
        name: value.skill_name,
        invocations: value.invocations,
        tokens: value.tokens.into(),
        projects: value.project_name.into_iter().collect(),
        last_used_at: value
            .last_used_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
        // Structured skill events are strong local correlation, not billing attribution.
        confidence: 0.85,
        source: MetricSource::from_internal(value.source, None),
    }
}

fn map_model(value: ModelRankItem) -> WireModelUsage {
    WireModelUsage {
        name: value.model,
        tokens: value.tokens.into(),
        requests: value.requests,
        projects: 0,
        cache_hit_ratio: value.cache_hit_ratio,
        source: MetricSource::from_internal(value.source, None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireSettings {
    pub mock_mode: bool,
    pub launch_at_startup: bool,
    pub show_widget_on_launch: bool,
    pub close_to_tray: bool,
    pub always_on_top: bool,
    pub capsule_mode: bool,
    pub lock_position: bool,
    pub auto_collapse: bool,
    pub show_in_taskbar: bool,
    pub theme: String,
    pub language: String,
    pub refresh_mode: String,
    pub custom_refresh_seconds: u64,
    pub notification_thresholds: Vec<u8>,
    pub primary_quota_window: String,
}

impl From<InternalSettings> for WireSettings {
    fn from(value: InternalSettings) -> Self {
        let (refresh_mode, custom_refresh_seconds) = match value.refresh_mode {
            RefreshMode::Smart => ("smart".into(), 30),
            RefreshMode::Fixed(10) => ("10s".into(), 10),
            RefreshMode::Fixed(30) => ("30s".into(), 30),
            RefreshMode::Fixed(60) => ("1m".into(), 60),
            RefreshMode::Fixed(300) => ("5m".into(), 300),
            RefreshMode::Fixed(seconds) => ("custom".into(), seconds),
        };
        Self {
            mock_mode: value.mock_mode,
            launch_at_startup: value.launch_at_login,
            show_widget_on_launch: value.show_floating_on_start,
            close_to_tray: value.hide_dashboard_on_close,
            always_on_top: value.always_on_top,
            capsule_mode: value.capsule_mode,
            lock_position: value.lock_widget_position,
            auto_collapse: value.auto_compact,
            show_in_taskbar: value.show_in_taskbar,
            theme: match value.theme {
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
                ThemeMode::System => "system",
            }
            .into(),
            language: match value.language {
                Language::ZhCn => "zh-CN",
                Language::En => "en",
            }
            .into(),
            refresh_mode,
            custom_refresh_seconds,
            notification_thresholds: value.notifications.remaining_thresholds,
            primary_quota_window: match value.primary_quota_window {
                QuotaWindowPreference::Primary => "primary",
                QuotaWindowPreference::Secondary => "secondary",
            }
            .into(),
        }
    }
}

impl WireSettings {
    pub fn apply_to(self, mut value: InternalSettings) -> Result<InternalSettings, String> {
        value.mock_mode = self.mock_mode;
        value.launch_at_login = self.launch_at_startup;
        value.show_floating_on_start = self.show_widget_on_launch;
        value.hide_dashboard_on_close = self.close_to_tray;
        value.always_on_top = self.always_on_top;
        value.capsule_mode = self.capsule_mode;
        value.lock_widget_position = self.lock_position;
        value.auto_compact = self.auto_collapse;
        value.show_in_taskbar = self.show_in_taskbar;
        value.theme = match self.theme.as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            "system" => ThemeMode::System,
            _ => return Err("theme must be light, dark, or system".into()),
        };
        value.language = match self.language.as_str() {
            "zh-CN" => Language::ZhCn,
            "en" => Language::En,
            _ => return Err("language must be zh-CN or en".into()),
        };
        value.refresh_mode = match self.refresh_mode.as_str() {
            "smart" => RefreshMode::Smart,
            "10s" => RefreshMode::Fixed(10),
            "30s" => RefreshMode::Fixed(30),
            "1m" => RefreshMode::Fixed(60),
            "5m" => RefreshMode::Fixed(300),
            "custom" => RefreshMode::Fixed(self.custom_refresh_seconds),
            _ => return Err("refreshMode is not supported".into()),
        };
        value.notifications.remaining_thresholds = self.notification_thresholds;
        value.primary_quota_window = match self.primary_quota_window.as_str() {
            "primary" => QuotaWindowPreference::Primary,
            "secondary" => QuotaWindowPreference::Secondary,
            _ => return Err("primaryQuotaWindow must be primary or secondary".into()),
        };
        Ok(value)
    }
}
