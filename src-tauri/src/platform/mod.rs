use serde::Serialize;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub platform: String,
    pub version: String,
    pub architecture: String,
    pub family: String,
}

pub fn info(_app: &AppHandle) -> PlatformInfo {
    PlatformInfo {
        platform: tauri_plugin_os::platform().to_string(),
        version: tauri_plugin_os::version().to_string(),
        architecture: tauri_plugin_os::arch().to_string(),
        family: tauri_plugin_os::family().to_string(),
    }
}
