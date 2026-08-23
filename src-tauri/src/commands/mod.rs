mod wire;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    analytics::{
        CacheAnalytics, DashboardAnalytics, ModelRankItem, ProjectDetail, ProjectRankItem,
        SkillRankItem, TrendPoint, UsageRange, UsageTotals,
    },
    app_state::{AppState, RefreshResult, RefreshStatus},
    error::{CommandError, CommandResult},
    platform::{self, PlatformInfo},
    quota::{CapabilityProbe, ProviderOutput},
    session::IngestReport,
    tray,
    window::{self, WindowTarget},
};

use wire::{WireOfficialResetVoucher, WireQuotaSnapshot, WireSettings, WireUsageSummary};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub settings: WireSettings,
    pub quota: WireQuotaSnapshot,
    pub official_reset_vouchers: Option<Vec<WireOfficialResetVoucher>>,
    pub official_reset_voucher_available_count: Option<u64>,
    pub usage: WireUsageSummary,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingQuery {
    #[serde(default)]
    pub range: UsageRange,
    #[serde(default = "default_limit")]
    pub limit: u16,
}

fn default_limit() -> u16 {
    50
}

#[tauri::command]
pub async fn bootstrap(state: State<'_, AppState>) -> CommandResult<BootstrapResponse> {
    let settings = state.settings().map_err(CommandError::from)?;
    // Bootstrap must be fast. The background scheduler refreshes stale quota
    // data and emits quota-updated when the fresh result is ready.
    let quota = state.current_quota().map_err(CommandError::from)?;
    let refresh = state.refresh_status().map_err(CommandError::from)?;
    let analytics = state
        .database
        .dashboard_analytics()
        .map_err(CommandError::from)?;
    let official_reset_vouchers = quota.snapshot.official_reset_credits.as_ref().map(|credits| {
        credits
            .credits
            .clone()
            .into_iter()
            .map(WireOfficialResetVoucher::from)
            .collect()
    });
    let official_reset_voucher_available_count = quota
        .snapshot
        .official_reset_credits
        .as_ref()
        .map(|credits| credits.available_count);
    Ok(BootstrapResponse {
        settings: settings.clone().into(),
        quota: WireQuotaSnapshot::from_output(quota, &refresh),
        official_reset_vouchers,
        official_reset_voucher_available_count,
        usage: if settings.mock_mode {
            WireUsageSummary::mock()
        } else {
            analytics.into()
        },
    })
}

#[tauri::command]
pub fn get_quota_snapshot(state: State<'_, AppState>) -> CommandResult<ProviderOutput> {
    state.current_quota().map_err(CommandError::from)
}

#[tauri::command]
pub fn get_quota_capabilities(state: State<'_, AppState>) -> CommandResult<CapabilityProbe> {
    Ok(state
        .current_quota()
        .map_err(CommandError::from)?
        .capabilities)
}

#[tauri::command]
pub async fn refresh_quota_snapshot(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<RefreshResult> {
    let seconds = state.recommended_refresh_seconds();
    let result = state
        .refresh_quota(seconds)
        .await
        .map_err(CommandError::from)?;
    let remaining = result
        .output
        .snapshot
        .secondary
        .as_ref()
        .or(result.output.snapshot.primary.as_ref())
        .map(|window| window.remaining_percent);
    tray::update_tooltip(&app, remaining);
    Ok(result)
}

#[tauri::command]
pub fn get_refresh_status(state: State<'_, AppState>) -> CommandResult<RefreshStatus> {
    state.refresh_status().map_err(CommandError::from)
}

#[tauri::command]
pub async fn sync_local_usage(state: State<'_, AppState>) -> CommandResult<IngestReport> {
    let database = state.database.clone();
    let report = tauri::async_runtime::spawn_blocking(move || {
        let ingestor = crate::session::SessionIngestor::discover();
        ingestor.ingest(&database)
    })
    .await
    .map_err(|error| CommandError {
        code: "ingest_task_failed",
        message: error.to_string(),
    })?
    .map_err(CommandError::from)?;
    Ok(report)
}

#[tauri::command]
pub fn clear_local_data(state: State<'_, AppState>) -> CommandResult<()> {
    state.clear_local_data().map_err(CommandError::from)
}

#[tauri::command]
pub fn get_dashboard_analytics(state: State<'_, AppState>) -> CommandResult<DashboardAnalytics> {
    state
        .database
        .dashboard_analytics()
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_usage_totals(
    state: State<'_, AppState>,
    range: UsageRange,
) -> CommandResult<UsageTotals> {
    state
        .database
        .usage_totals(range)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_usage_trend(
    state: State<'_, AppState>,
    range: UsageRange,
) -> CommandResult<Vec<TrendPoint>> {
    state
        .database
        .usage_trend(range)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_projects(
    state: State<'_, AppState>,
    query: RankingQuery,
) -> CommandResult<Vec<ProjectRankItem>> {
    state
        .database
        .projects(query.range, query.limit)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_project_detail(
    state: State<'_, AppState>,
    project_id: String,
    range: UsageRange,
) -> CommandResult<Option<ProjectDetail>> {
    state
        .database
        .project_detail(&project_id, range)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_skills(
    state: State<'_, AppState>,
    query: RankingQuery,
) -> CommandResult<Vec<SkillRankItem>> {
    state
        .database
        .skills(query.range, query.limit)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_models(
    state: State<'_, AppState>,
    query: RankingQuery,
) -> CommandResult<Vec<ModelRankItem>> {
    state
        .database
        .models(query.range, query.limit)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_cache_analytics(
    state: State<'_, AppState>,
    range: UsageRange,
) -> CommandResult<CacheAnalytics> {
    state
        .database
        .cache_analytics(range)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<WireSettings> {
    state
        .settings()
        .map(WireSettings::from)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: WireSettings,
) -> CommandResult<WireSettings> {
    let previous = state.settings().map_err(CommandError::from)?;
    let mut settings = settings
        .apply_to(previous.clone())
        .map_err(|message| CommandError {
            code: "invalid_setting",
            message,
        })?;
    settings = settings.validate().map_err(CommandError::from)?;
    if let Ok((x, y)) = window::current_widget_position(&app) {
        settings.widget_x = Some(x);
        settings.widget_y = Some(y);
    }
    if settings.launch_at_login != previous.launch_at_login {
        let autostart = app.autolaunch();
        if settings.launch_at_login {
            autostart.enable()
        } else {
            autostart.disable()
        }
        .map_err(|error| CommandError {
            code: "autostart_failed",
            message: error.to_string(),
        })?;
    }
    let internal_saved = state
        .update_settings(settings)
        .map_err(CommandError::from)?;
    window::apply_settings(&app, &internal_saved).map_err(CommandError::from)?;
    if internal_saved.capsule_mode != previous.capsule_mode
        || internal_saved.lock_widget_position != previous.lock_widget_position
    {
        window::set_compact(
            &app,
            internal_saved.capsule_mode && !internal_saved.lock_widget_position,
        )
        .map_err(CommandError::from)?;
    }
    let saved = WireSettings::from(internal_saved);
    tray::update_always_on_top_check(&app, saved.always_on_top);
    let _ = app.emit("my-codex://settings-updated", saved.clone());
    Ok(saved)
}

#[tauri::command]
pub fn save_widget_position(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<WireSettings> {
    let (x, y) = window::current_widget_position(&app).map_err(CommandError::from)?;
    let mut settings = state.settings().map_err(CommandError::from)?;
    settings.widget_x = Some(x);
    settings.widget_y = Some(y);
    state
        .update_settings(settings)
        .map(WireSettings::from)
        .map_err(CommandError::from)
}

#[tauri::command]
pub fn show_window(
    app: AppHandle,
    target: WindowTarget,
    route: Option<String>,
) -> CommandResult<()> {
    window::show(&app, target).map_err(CommandError::from)?;
    if let Some(route) = route.filter(|value| value.len() <= 64) {
        app.emit_to("dashboard", "my-codex://navigate", route)
            .map_err(|error| CommandError {
                code: "navigation_failed",
                message: error.to_string(),
            })?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle, target: WindowTarget) -> CommandResult<()> {
    window::hide(&app, target).map_err(CommandError::from)
}

#[tauri::command]
pub fn toggle_window(app: AppHandle, target: WindowTarget) -> CommandResult<()> {
    match target {
        WindowTarget::Dashboard => window::toggle_maximized(&app, target),
        WindowTarget::Floating => window::toggle(&app, target),
    }
    .map_err(CommandError::from)
}

#[tauri::command]
pub fn set_floating_compact(app: AppHandle, compact: bool) -> CommandResult<()> {
    window::set_compact(&app, compact).map_err(CommandError::from)
}

#[tauri::command]
pub fn test_notification(state: State<'_, AppState>) -> CommandResult<()> {
    state.notify_test().map_err(CommandError::from)
}

#[tauri::command]
pub fn get_platform_info(app: AppHandle) -> PlatformInfo {
    platform::info(&app)
}

#[tauri::command]
pub fn exit_application(app: AppHandle) {
    app.exit(0);
}
