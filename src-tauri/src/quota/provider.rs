use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

use super::{ProviderKind, QuotaSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityState {
    Supported,
    Unsupported,
    AuthUnavailable,
    TemporarilyUnavailable,
    Incompatible,
    NotProbed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityProbe {
    pub account_read: CapabilityState,
    pub rate_limits_read: CapabilityState,
    pub usage_read: CapabilityState,
}

impl Default for CapabilityProbe {
    fn default() -> Self {
        Self {
            account_read: CapabilityState::NotProbed,
            rate_limits_read: CapabilityState::NotProbed,
            usage_read: CapabilityState::NotProbed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderOutput {
    pub provider: ProviderKind,
    pub capabilities: CapabilityProbe,
    pub snapshot: QuotaSnapshot,
}

#[async_trait]
pub trait QuotaProvider: Send + Sync {
    async fn fetch(&self) -> AppResult<ProviderOutput>;
}

