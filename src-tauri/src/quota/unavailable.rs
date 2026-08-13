use async_trait::async_trait;

use crate::error::AppResult;

use super::{
    CapabilityProbe, CapabilityState, ProviderKind, ProviderOutput, QuotaProvider,
    QuotaSnapshot,
};

pub struct UnavailableQuotaProvider;

#[async_trait]
impl QuotaProvider for UnavailableQuotaProvider {
    async fn fetch(&self) -> AppResult<ProviderOutput> {
        Ok(ProviderOutput {
            provider: ProviderKind::Unavailable,
            capabilities: CapabilityProbe {
                account_read: CapabilityState::Unsupported,
                rate_limits_read: CapabilityState::Unsupported,
                usage_read: CapabilityState::Unsupported,
            },
            snapshot: QuotaSnapshot::unavailable(
                ProviderKind::Unavailable,
                "未检测到支持 app-server 的 Codex CLI；未读取认证文件，也未生成模拟额度。",
            ),
        })
    }
}
