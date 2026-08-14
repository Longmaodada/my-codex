use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    ZhCn,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuotaWindowPreference {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode", content = "seconds")]
pub enum RefreshMode {
    Smart,
    Fixed(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub remaining_thresholds: Vec<u8>,
    pub notify_on_unavailable: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            remaining_thresholds: vec![50, 25, 20, 10, 5],
            notify_on_unavailable: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub schema_version: u32,
    pub theme: ThemeMode,
    pub language: Language,
    pub launch_at_login: bool,
    pub show_floating_on_start: bool,
    pub hide_dashboard_on_close: bool,
    pub always_on_top: bool,
    pub capsule_mode: bool,
    pub lock_widget_position: bool,
    pub auto_compact: bool,
    pub compact_after_seconds: u64,
    pub show_in_taskbar: bool,
    pub refresh_mode: RefreshMode,
    pub mock_mode: bool,
    pub widget_x: Option<i32>,
    pub widget_y: Option<i32>,
    pub retention_days: u16,
    pub notifications: NotificationSettings,
    pub healthy_remaining_percent: u8,
    pub warning_remaining_percent: u8,
    pub primary_quota_window: QuotaWindowPreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            theme: ThemeMode::System,
            language: Language::ZhCn,
            launch_at_login: false,
            show_floating_on_start: true,
            hide_dashboard_on_close: true,
            always_on_top: true,
            capsule_mode: true,
            lock_widget_position: false,
            auto_compact: true,
            compact_after_seconds: 30,
            show_in_taskbar: false,
            refresh_mode: RefreshMode::Smart,
            mock_mode: false,
            widget_x: None,
            widget_y: None,
            retention_days: 365,
            notifications: NotificationSettings::default(),
            healthy_remaining_percent: 40,
            warning_remaining_percent: 20,
            primary_quota_window: QuotaWindowPreference::Secondary,
        }
    }
}

impl AppSettings {
    pub fn validate(mut self) -> AppResult<Self> {
        if !(5..=86_400).contains(&self.compact_after_seconds) {
            return Err(AppError::InvalidSetting(
                "compactAfterSeconds must be between 5 and 86400".into(),
            ));
        }

        if !(30..=3650).contains(&self.retention_days) {
            return Err(AppError::InvalidSetting(
                "retentionDays must be between 30 and 3650".into(),
            ));
        }

        if self.warning_remaining_percent >= self.healthy_remaining_percent
            || self.healthy_remaining_percent > 100
        {
            return Err(AppError::InvalidSetting(
                "remaining percentage thresholds are out of order".into(),
            ));
        }

        if let RefreshMode::Fixed(seconds) = self.refresh_mode {
            if !(10..=3600).contains(&seconds) {
                return Err(AppError::InvalidSetting(
                    "fixed refresh interval must be between 10 and 3600 seconds".into(),
                ));
            }
        }

        self.notifications
            .remaining_thresholds
            .retain(|value| (1..=100).contains(value));
        self.notifications
            .remaining_thresholds
            .sort_unstable_by(|a, b| b.cmp(a));
        self.notifications.remaining_thresholds.dedup();
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_notification_thresholds() {
        let mut settings = AppSettings::default();
        settings.notifications.remaining_thresholds = vec![10, 120, 10, 25, 0];
        let settings = settings.validate().expect("valid settings");
        assert_eq!(settings.notifications.remaining_thresholds, vec![25, 10]);
    }

    #[test]
    fn rejects_aggressive_polling() {
        let mut settings = AppSettings::default();
        settings.refresh_mode = RefreshMode::Fixed(2);
        assert!(settings.validate().is_err());
    }
}
