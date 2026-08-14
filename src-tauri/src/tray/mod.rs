use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager,
};

use crate::{
    app_state::AppState,
    error::{AppError, AppResult},
    window::{self, WindowTarget},
};

pub struct TrayMenuState {
    always_on_top: CheckMenuItem<tauri::Wry>,
}

pub fn setup(app: &App) -> AppResult<()> {
    let always_on_top_enabled = app.state::<AppState>().settings()?.always_on_top;
    let floating = MenuItem::with_id(app, "floating", "显示/隐藏悬浮窗", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let dashboard = MenuItem::with_id(app, "dashboard", "打开主面板", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let refresh = MenuItem::with_id(app, "refresh", "立即刷新", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let always_on_top = CheckMenuItem::with_id(
        app,
        "always_on_top",
        "始终置顶",
        true,
        always_on_top_enabled,
        None::<&str>,
    )
    .map_err(|error| AppError::Other(error.to_string()))?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let about = MenuItem::with_id(app, "about", "关于", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let separator =
        PredefinedMenuItem::separator(app).map_err(|error| AppError::Other(error.to_string()))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|error| AppError::Other(error.to_string()))?;
    let menu = Menu::with_items(
        app,
        &[
            &floating,
            &dashboard,
            &refresh,
            &always_on_top,
            &settings,
            &about,
            &separator,
            &quit,
        ],
    )
    .map_err(|error| AppError::Other(error.to_string()))?;
    let mut builder = TrayIconBuilder::with_id("my-codex-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("My Codex · 额度等待刷新")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "floating" => {
                let _ = window::toggle(app, WindowTarget::Floating);
            }
            "dashboard" => {
                let _ = window::show(app, WindowTarget::Dashboard);
            }
            "refresh" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let interval = handle.state::<AppState>().recommended_refresh_seconds();
                    if let Ok(result) = handle.state::<AppState>().refresh_quota(interval).await {
                        let remaining = result
                            .output
                            .snapshot
                            .secondary
                            .as_ref()
                            .or(result.output.snapshot.primary.as_ref())
                            .map(|window| window.remaining_percent);
                        update_tooltip(&handle, remaining);
                        let _ = handle.emit("my-codex://quota-updated", result);
                    }
                });
            }
            "always_on_top" => {
                if let Ok(mut settings) = app.state::<AppState>().settings() {
                    let enabled = !settings.always_on_top;
                    settings.always_on_top = enabled;
                    if app.state::<AppState>().update_settings(settings).is_ok() {
                        if window::set_always_on_top(app, enabled).is_ok() {
                            update_always_on_top_check(app, enabled);
                            let _ = app.emit("my-codex://settings-updated", ());
                        }
                    }
                }
            }
            "settings" => {
                let _ = window::show(app, WindowTarget::Dashboard);
                let _ = app.emit_to("dashboard", "my-codex://navigate", "settings");
            }
            "about" => {
                let _ = window::show(app, WindowTarget::Dashboard);
                let _ = app.emit_to("dashboard", "my-codex://navigate", "about");
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                let _ = window::toggle(tray.app_handle(), WindowTarget::Floating);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    } else {
        builder = builder.icon(fallback_icon());
    }
    builder
        .build(app)
        .map_err(|error| AppError::Other(error.to_string()))?;
    app.manage(TrayMenuState { always_on_top });
    Ok(())
}

pub fn update_always_on_top_check(app: &tauri::AppHandle, enabled: bool) {
    if let Some(menu) = app.try_state::<TrayMenuState>() {
        let _ = menu.always_on_top.set_checked(enabled);
    }
}

fn fallback_icon() -> tauri::image::Image<'static> {
    const SIZE: u32 = 32;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as i32 - 15;
            let dy = y as i32 - 15;
            let inside = dx * dx + dy * dy <= 14 * 14;
            let blend = ((x + y) * 255 / ((SIZE - 1) * 2)) as u8;
            rgba.extend_from_slice(&[
                82_u8.saturating_add(blend / 5),
                98_u8.saturating_sub(blend / 12),
                232_u8.saturating_add(blend / 12),
                if inside { 255 } else { 0 },
            ]);
        }
    }
    tauri::image::Image::new_owned(rgba, SIZE, SIZE)
}

pub fn update_tooltip(app: &tauri::AppHandle, remaining_percent: Option<f64>) {
    if let Some(tray) = app.tray_by_id("my-codex-tray") {
        let text = remaining_percent
            .map(|value| format!("My Codex · 剩余 {:.0}%", value.clamp(0.0, 100.0)))
            .unwrap_or_else(|| "My Codex · 数据不可用".into());
        let _ = tray.set_tooltip(Some(text));
    }
}
