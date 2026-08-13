mod app_server;
mod mock;
mod provider;
mod unavailable;

use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::{Deserialize, Serialize};

use crate::{
    analytics::{DataSource, TokenBreakdown},
    error::AppResult,
};

pub use provider::{CapabilityProbe, CapabilityState, ProviderOutput, QuotaProvider};

use app_server::AppServerProvider;
use mock::MockQuotaProvider;
use unavailable::UnavailableQuotaProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    AppServer,
    Mock,
    Unavailable,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppServer => "app_server",
            Self::Mock => "mock",
            Self::Unavailable => "unavailable",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "app_server" => Self::AppServer,
            "mock" => Self::Mock,
            _ => Self::Unavailable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuotaStatus {
    Available,
    Stale,
    SignedOut,
    Unavailable,
    Incompatible,
}

impl QuotaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Stale => "stale",
            Self::SignedOut => "signed_out",
            Self::Unavailable => "unavailable",
            Self::Incompatible => "incompatible",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "available" => Self::Available,
            "stale" => Self::Stale,
            "signed_out" => Self::SignedOut,
            "incompatible" => Self::Incompatible,
            _ => Self::Unavailable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub window_duration_minutes: Option<i64>,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialUsageSummary {
    pub lifetime_tokens: Option<i64>,
    pub peak_daily_tokens: Option<i64>,
    pub longest_running_turn_seconds: Option<i64>,
    pub current_streak_days: Option<i64>,
    pub longest_streak_days: Option<i64>,
    pub daily_usage: Vec<OfficialDailyUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialDailyUsage {
    pub start_date: String,
    pub tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    pub captured_at: DateTime<Utc>,
    pub provider: ProviderKind,
    pub status: QuotaStatus,
    pub source: DataSource,
    pub plan: Option<String>,
    pub primary: Option<QuotaWindow>,
    pub secondary: Option<QuotaWindow>,
    pub reset_at: Option<DateTime<Utc>>,
    pub tokens: Option<TokenBreakdown>,
    pub official_usage: Option<OfficialUsageSummary>,
    pub message: Option<String>,
}

impl QuotaSnapshot {
    pub fn unavailable(provider: ProviderKind, message: impl Into<String>) -> Self {
        Self {
            captured_at: Utc::now(),
            provider,
            status: QuotaStatus::Unavailable,
            source: DataSource::Official,
            plan: None,
            primary: None,
            secondary: None,
            reset_at: None,
            tokens: None,
            official_usage: None,
            message: Some(message.into()),
        }
    }

    pub(crate) fn from_database_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let captured_at: String = row.get(0)?;
        let provider: String = row.get(1)?;
        let status: String = row.get(2)?;
        let source: String = row.get(3)?;
        let primary_reset_at: Option<String> = row.get(8)?;
        let secondary_reset_at: Option<String> = row.get(12)?;
        let reset_at: Option<String> = row.get(13)?;
        let input: Option<i64> = row.get(14)?;
        let output: Option<i64> = row.get(15)?;
        let cached: Option<i64> = row.get(16)?;
        let reasoning: Option<i64> = row.get(17)?;
        let total: Option<i64> = row.get(18)?;
        let parse_time = |value: &str| {
            DateTime::parse_from_rfc3339(value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(error),
                    )
                })
        };
        let primary_used: Option<f64> = row.get(5)?;
        let primary_remaining: Option<f64> = row.get(6)?;
        let primary_window_minutes: Option<i64> = row.get(7)?;
        let secondary_used: Option<f64> = row.get(9)?;
        let secondary_remaining: Option<f64> = row.get(10)?;
        let secondary_window_minutes: Option<i64> = row.get(11)?;
        Ok(Self {
            captured_at: parse_time(&captured_at)?,
            provider: ProviderKind::from_str(&provider),
            status: QuotaStatus::from_str(&status),
            source: match source.as_str() {
                "local" => DataSource::Local,
                "estimated" => DataSource::Estimated,
                _ => DataSource::Official,
            },
            plan: row.get(4)?,
            primary: primary_used.map(|used_percent| QuotaWindow {
                used_percent,
                remaining_percent: primary_remaining.unwrap_or(100.0 - used_percent),
                window_duration_minutes: primary_window_minutes,
                resets_at: primary_reset_at.as_deref().and_then(parse_optional_time),
            }),
            secondary: secondary_used.map(|used_percent| QuotaWindow {
                used_percent,
                remaining_percent: secondary_remaining.unwrap_or(100.0 - used_percent),
                window_duration_minutes: secondary_window_minutes,
                resets_at: secondary_reset_at.as_deref().and_then(parse_optional_time),
            }),
            reset_at: reset_at.as_deref().and_then(parse_optional_time),
            tokens: input.map(|input_tokens| {
                TokenBreakdown {
                    input_tokens,
                    output_tokens: output.unwrap_or_default(),
                    cached_input_tokens: cached.unwrap_or_default(),
                    reasoning_tokens: reasoning.unwrap_or_default(),
                    total_tokens: total.unwrap_or_default(),
                }
                .normalize()
            }),
            official_usage: None,
            message: None,
        })
    }
}

fn parse_optional_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

pub fn epoch_seconds(value: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(value, 0).single()
}

pub struct QuotaService {
    app_server: AppServerProvider,
    mock: MockQuotaProvider,
    unavailable: UnavailableQuotaProvider,
}

impl QuotaService {
    pub fn new(codex_binary: Option<PathBuf>) -> Self {
        Self {
            app_server: AppServerProvider::new(codex_binary),
            mock: MockQuotaProvider,
            unavailable: UnavailableQuotaProvider,
        }
    }

    pub async fn fetch(&self, mock_mode: bool) -> AppResult<ProviderOutput> {
        if mock_mode {
            return self.mock.fetch().await;
        }
        if self.app_server.is_available() {
            self.app_server.fetch().await
        } else {
            self.unavailable.fetch().await
        }
    }
}
