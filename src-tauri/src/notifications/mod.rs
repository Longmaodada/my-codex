use std::{collections::HashSet, sync::Mutex};

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::{
    error::{AppError, AppResult},
    quota::{QuotaSnapshot, QuotaStatus},
    settings::NotificationSettings,
};

pub struct NotificationController {
    sent_thresholds: Mutex<HashSet<u8>>,
}

impl Default for NotificationController {
    fn default() -> Self {
        Self {
            sent_thresholds: Mutex::new(HashSet::new()),
        }
    }
}

impl NotificationController {
    pub fn evaluate(
        &self,
        app: &AppHandle,
        settings: &NotificationSettings,
        quota: &QuotaSnapshot,
    ) -> AppResult<()> {
        if !settings.enabled || quota.status != QuotaStatus::Available {
            return Ok(());
        }
        let Some(remaining) = quota
            .secondary
            .as_ref()
            .or(quota.primary.as_ref())
            .map(|window| window.remaining_percent.clamp(0.0, 100.0))
        else {
            return Ok(());
        };
        let mut sent = self
            .sent_thresholds
            .lock()
            .map_err(|_| AppError::StatePoisoned)?;
        sent.retain(|threshold| remaining <= f64::from(*threshold));
        let threshold = settings
            .remaining_thresholds
            .iter()
            .copied()
            .filter(|threshold| remaining <= f64::from(*threshold))
            .min();
        let Some(threshold) = threshold else {
            return Ok(());
        };
        if !sent.insert(threshold) {
            return Ok(());
        }
        app.notification()
            .builder()
            .title("My Codex 额度提醒")
            .body(format!("Codex 额度剩余约 {:.0}%", remaining))
            .show()
            .map_err(|error| AppError::Notification(error.to_string()))?;
        Ok(())
    }

    pub fn test(&self, app: &AppHandle) -> AppResult<()> {
        app.notification()
            .builder()
            .title("My Codex")
            .body("通知已启用。")
            .show()
            .map_err(|error| AppError::Notification(error.to_string()))
    }
}
