use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("local I/O operation failed")]
    Io(#[from] std::io::Error),
    #[error("local database operation failed")]
    Database(#[from] rusqlite::Error),
    #[error("local data could not be parsed")]
    Json(#[from] serde_json::Error),
    #[error("invalid setting: {0}")]
    InvalidSetting(String),
    #[error("Codex quota provider is unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("window operation failed: {0}")]
    Window(String),
    #[error("notification operation failed: {0}")]
    Notification(String),
    #[error("application state lock is poisoned")]
    StatePoisoned,
    #[error("operation failed: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(value: AppError) -> Self {
        let code = match &value {
            AppError::Io(_) => "local_io_failed",
            AppError::Database(_) => "database_failed",
            AppError::Json(_) => "local_data_invalid",
            AppError::InvalidSetting(_) => "invalid_setting",
            AppError::ProviderUnavailable(_) => "quota_unavailable",
            AppError::Window(_) => "window_failed",
            AppError::Notification(_) => "notification_failed",
            AppError::StatePoisoned => "state_unavailable",
            AppError::Other(_) => "operation_failed",
        };

        Self {
            code,
            message: value.to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type CommandResult<T> = Result<T, CommandError>;
