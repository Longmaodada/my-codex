use serde::{Deserialize, Serialize};
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, Position, Size};

use crate::{
    error::{AppError, AppResult},
    settings::AppSettings,
};

const FLOATING_EXPANDED_WIDTH: f64 = 320.0;
const FLOATING_EXPANDED_HEIGHT: f64 = 500.0;
const FLOATING_COMPACT_WIDTH: f64 = 140.0;
const FLOATING_COMPACT_HEIGHT: f64 = 56.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowTarget {
    Floating,
    Dashboard,
}

impl WindowTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Floating => "floating",
            Self::Dashboard => "dashboard",
        }
    }
}

pub fn show(app: &AppHandle, target: WindowTarget) -> AppResult<()> {
    let window = app
        .get_webview_window(target.label())
        .ok_or_else(|| AppError::Window(format!("{} window is missing", target.label())))?;
    window
        .show()
        .map_err(|error| AppError::Window(error.to_string()))?;
    if matches!(target, WindowTarget::Dashboard) {
        window
            .set_focus()
            .map_err(|error| AppError::Window(error.to_string()))?;
    }
    Ok(())
}

pub fn hide(app: &AppHandle, target: WindowTarget) -> AppResult<()> {
    app.get_webview_window(target.label())
        .ok_or_else(|| AppError::Window(format!("{} window is missing", target.label())))?
        .hide()
        .map_err(|error| AppError::Window(error.to_string()))
}

pub fn toggle(app: &AppHandle, target: WindowTarget) -> AppResult<()> {
    let window = app
        .get_webview_window(target.label())
        .ok_or_else(|| AppError::Window(format!("{} window is missing", target.label())))?;
    if window
        .is_visible()
        .map_err(|error| AppError::Window(error.to_string()))?
    {
        window
            .hide()
            .map_err(|error| AppError::Window(error.to_string()))
    } else {
        window
            .show()
            .map_err(|error| AppError::Window(error.to_string()))?;
        window
            .set_focus()
            .map_err(|error| AppError::Window(error.to_string()))
    }
}

pub fn toggle_maximized(app: &AppHandle, target: WindowTarget) -> AppResult<()> {
    let window = app
        .get_webview_window(target.label())
        .ok_or_else(|| AppError::Window(format!("{} window is missing", target.label())))?;
    let maximized = window
        .is_maximized()
        .map_err(|error| AppError::Window(error.to_string()))?;
    if maximized {
        window
            .unmaximize()
            .map_err(|error| AppError::Window(error.to_string()))
    } else {
        window
            .maximize()
            .map_err(|error| AppError::Window(error.to_string()))
    }
}

pub fn set_compact(app: &AppHandle, compact: bool) -> AppResult<()> {
    let window = app
        .get_webview_window("floating")
        .ok_or_else(|| AppError::Window("floating window is missing".into()))?;
    let (width, height) = if compact {
        (FLOATING_COMPACT_WIDTH, FLOATING_COMPACT_HEIGHT)
    } else {
        (FLOATING_EXPANDED_WIDTH, FLOATING_EXPANDED_HEIGHT)
    };
    window
        .set_size(Size::Logical(LogicalSize::new(width, height)))
        .map_err(|error| AppError::Window(error.to_string()))
}

pub fn set_always_on_top(app: &AppHandle, enabled: bool) -> AppResult<()> {
    app.get_webview_window("floating")
        .ok_or_else(|| AppError::Window("floating window is missing".into()))?
        .set_always_on_top(enabled)
        .map_err(|error| AppError::Window(error.to_string()))
}

pub fn apply_settings(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    let floating = app
        .get_webview_window("floating")
        .ok_or_else(|| AppError::Window("floating window is missing".into()))?;
    floating
        .set_always_on_top(settings.always_on_top)
        .map_err(|error| AppError::Window(error.to_string()))?;
    #[cfg(not(target_os = "macos"))]
    floating
        .set_skip_taskbar(true)
        .map_err(|error| AppError::Window(error.to_string()))?;
    let dashboard = app
        .get_webview_window("dashboard")
        .ok_or_else(|| AppError::Window("dashboard window is missing".into()))?;
    #[cfg(not(target_os = "macos"))]
    dashboard
        .set_skip_taskbar(true)
        .map_err(|error| AppError::Window(error.to_string()))?;
    if let (Some(x), Some(y)) = (settings.widget_x, settings.widget_y) {
        let (x, y) = safe_widget_position(app, &floating, x, y)?;
        floating
            .set_position(Position::Physical(PhysicalPosition::new(x, y)))
            .map_err(|error| AppError::Window(error.to_string()))?;
    }
    if settings.show_floating_on_start {
        floating
            .show()
            .map_err(|error| AppError::Window(error.to_string()))?;
    } else {
        floating
            .hide()
            .map_err(|error| AppError::Window(error.to_string()))?;
    }
    Ok(())
}

fn safe_widget_position(
    app: &AppHandle,
    floating: &tauri::WebviewWindow,
    x: i32,
    y: i32,
) -> AppResult<(i32, i32)> {
    let size = floating
        .outer_size()
        .map_err(|error| AppError::Window(error.to_string()))?;
    let width = i64::from(size.width);
    let height = i64::from(size.height);
    let x = i64::from(x);
    let y = i64::from(y);

    let monitors = app
        .available_monitors()
        .map_err(|error| AppError::Window(error.to_string()))?;
    let visible = monitors.iter().any(|monitor| {
        let area = monitor.work_area();
        let left = i64::from(area.position.x);
        let top = i64::from(area.position.y);
        let right = left + i64::from(area.size.width);
        let bottom = top + i64::from(area.size.height);
        x < right && x + width > left && y < bottom && y + height > top
    });
    if visible {
        return Ok((x as i32, y as i32));
    }

    let monitor = app
        .primary_monitor()
        .map_err(|error| AppError::Window(error.to_string()))?
        .or_else(|| monitors.first().cloned())
        .ok_or_else(|| AppError::Window("no monitor is available".into()))?;
    let area = monitor.work_area();
    let right = i64::from(area.position.x) + i64::from(area.size.width);
    let bottom = i64::from(area.position.y) + i64::from(area.size.height);
    let next_x = (right - width - 24).max(i64::from(area.position.x));
    let next_y =
        (i64::from(area.position.y) + 24).min((bottom - height).max(i64::from(area.position.y)));
    Ok((next_x as i32, next_y as i32))
}

pub fn current_widget_position(app: &AppHandle) -> AppResult<(i32, i32)> {
    let position = app
        .get_webview_window("floating")
        .ok_or_else(|| AppError::Window("floating window is missing".into()))?
        .outer_position()
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok((position.x, position.y))
}

pub fn install_close_to_tray(app: &AppHandle) -> AppResult<()> {
    let dashboard = app
        .get_webview_window("dashboard")
        .ok_or_else(|| AppError::Window("dashboard window is missing".into()))?;
    let window_to_hide = dashboard.clone();
    let state_app = app.clone();
    dashboard.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            let close_to_tray = state_app
                .try_state::<crate::app_state::AppState>()
                .and_then(|state| state.settings().ok())
                .map(|settings| settings.hide_dashboard_on_close)
                .unwrap_or(true);
            if close_to_tray {
                api.prevent_close();
                let _ = window_to_hide.hide();
            }
        }
    });
    Ok(())
}
