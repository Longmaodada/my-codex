use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::{
    analytics::{DataSource, TokenBreakdown},
    error::AppResult,
};

use super::{
    CapabilityProbe, CapabilityState, ProviderKind, ProviderOutput, QuotaProvider, QuotaSnapshot,
    QuotaStatus, QuotaWindow,
};

pub struct MockQuotaProvider;

#[async_trait]
impl QuotaProvider for MockQuotaProvider {
    async fn fetch(&self) -> AppResult<ProviderOutput> {
        let primary_reset_at = Utc::now() + Duration::hours(2) + Duration::minutes(18);
        let secondary_reset_at = Utc::now() + Duration::days(2) + Duration::hours(13);
        Ok(ProviderOutput {
            provider: ProviderKind::Mock,
            capabilities: CapabilityProbe {
                account_read: CapabilityState::Supported,
                rate_limits_read: CapabilityState::Supported,
                usage_read: CapabilityState::Supported,
            },
            snapshot: QuotaSnapshot {
                captured_at: Utc::now(),
                provider: ProviderKind::Mock,
                status: QuotaStatus::Available,
                source: DataSource::Estimated,
                plan: Some("Codex Pro · Mock".into()),
                primary: Some(QuotaWindow {
                    used_percent: 14.0,
                    remaining_percent: 86.0,
                    window_duration_minutes: Some(300),
                    resets_at: Some(primary_reset_at),
                }),
                secondary: Some(QuotaWindow {
                    used_percent: 27.0,
                    remaining_percent: 73.0,
                    window_duration_minutes: Some(10_080),
                    resets_at: Some(secondary_reset_at),
                }),
                reset_at: Some(secondary_reset_at),
                tokens: Some(TokenBreakdown {
                    input_tokens: 2_030_000_000,
                    output_tokens: 430_000_000,
                    cached_input_tokens: 320_000_000,
                    reasoning_tokens: 440_000_000,
                    total_tokens: 2_900_000_000,
                }),
                official_usage: None,
                message: Some("Mock Mode：仅用于界面演示，不是官方额度或账单数据。".into()),
            },
        })
    }
}
