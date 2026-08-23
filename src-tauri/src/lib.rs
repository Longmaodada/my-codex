mod analytics;
mod app_state;
mod codex;
mod commands;
mod database;
mod error;
mod notifications;
mod platform;
mod quota;
mod session;
mod settings;
mod tray;
mod window;

use std::time::Duration;

use app_state::AppState;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::get_quota_snapshot,
            commands::get_quota_capabilities,
            commands::refresh_quota_snapshot,
            commands::get_refresh_status,
            commands::sync_local_usage,
            commands::clear_local_data,
            commands::get_dashboard_analytics,
            commands::get_usage_totals,
            commands::get_usage_trend,
            commands::get_projects,
            commands::get_project_detail,
            commands::get_skills,
            commands::get_models,
            commands::get_cache_analytics,
            commands::get_settings,
            commands::save_settings,
            commands::save_widget_position,
            commands::show_window,
            commands::hide_window,
            commands::toggle_window,
            commands::set_floating_compact,
            commands::test_notification,
            commands::get_platform_info,
            commands::exit_application,
        ])
        .setup(|app| {
            let state = AppState::initialize(app.handle())?;
            let settings = state.settings()?;
            app.manage(state);

            window::apply_settings(app.handle(), &settings)?;
            window::set_compact(
                app.handle(),
                settings.capsule_mode && !settings.lock_widget_position,
            )?;
            window::install_close_to_tray(app.handle())?;
            tray::setup(app)?;
            setup_global_shortcuts(app)?;
            start_background_tasks(app.handle().clone());
            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("My Codex failed to start");
}

fn setup_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    let primary_modifier = Modifiers::META;
    #[cfg(not(target_os = "macos"))]
    let primary_modifier = Modifiers::CONTROL;
    let modifiers = primary_modifier | Modifiers::SHIFT;
    let toggle_widget = Shortcut::new(Some(modifiers), Code::KeyQ);
    let open_dashboard = Shortcut::new(Some(modifiers), Code::KeyD);
    let widget_for_handler = toggle_widget.clone();
    let dashboard_for_handler = open_dashboard.clone();

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                if shortcut == &widget_for_handler {
                    let _ = window::toggle(app, window::WindowTarget::Floating);
                } else if shortcut == &dashboard_for_handler {
                    let _ = window::show(app, window::WindowTarget::Dashboard);
                }
            })
            .build(),
    )?;
    // A shortcut may already belong to another application; that must not prevent startup.
    let _ = app.global_shortcut().register(toggle_widget);
    let _ = app.global_shortcut().register(open_dashboard);
    Ok(())
}

fn start_background_tasks(app: tauri::AppHandle) {
    let ingest_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let database = ingest_app.state::<AppState>().database.clone();
        let ingest_result = tauri::async_runtime::spawn_blocking(move || {
            let ingestor = session::SessionIngestor::discover();
            let report = ingestor.ingest(&database)?;
            let settings = database.load_settings()?;
            let _ = database.prune(settings.retention_days)?;
            Ok::<_, error::AppError>(report)
        })
        .await;
        if let Ok(Ok(report)) = ingest_result {
            let _ = ingest_app.emit("my-codex://usage-updated", report);
        }
    });

    let usage_app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let database = usage_app.state::<AppState>().database.clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                session::SessionIngestor::discover().ingest(&database)
            })
            .await;
            if let Ok(Ok(report)) = result {
                if report.sessions_upserted > 0 {
                    let _ = usage_app.emit("my-codex://usage-updated", report);
                }
            }
        }
    });

    tauri::async_runtime::spawn(async move {
        loop {
            let should_refresh = app.state::<AppState>().refresh_due();
            if should_refresh {
                let interval = app.state::<AppState>().recommended_refresh_seconds();
                if let Ok(result) = app.state::<AppState>().refresh_quota(interval).await {
                    let remaining = result
                        .output
                        .snapshot
                        .secondary
                        .as_ref()
                        .or(result.output.snapshot.primary.as_ref())
                        .map(|window| window.remaining_percent);
                    tray::update_tooltip(&app, remaining);
                    let _ = app.emit("my-codex://quota-updated", result);
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}
