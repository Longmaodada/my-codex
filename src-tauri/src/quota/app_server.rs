use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::timeout,
};

use crate::{
    analytics::DataSource,
    error::{AppError, AppResult},
};

use super::{
    epoch_seconds, CapabilityProbe, CapabilityState, OfficialDailyUsage, OfficialUsageSummary,
    ProviderKind, ProviderOutput, QuotaProvider, QuotaSnapshot, QuotaStatus, QuotaWindow,
};

const RPC_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_JSON_LINE_BYTES: usize = 1024 * 1024;

pub struct AppServerProvider {
    executable: Option<PathBuf>,
}

impl AppServerProvider {
    pub fn new(configured: Option<PathBuf>) -> Self {
        Self {
            executable: find_codex_executable(configured),
        }
    }

    pub fn is_available(&self) -> bool {
        self.executable.is_some()
    }

    async fn fetch_inner(&self) -> AppResult<ProviderOutput> {
        let executable = self
            .executable
            .as_ref()
            .ok_or_else(|| AppError::ProviderUnavailable("Codex CLI was not found".into()))?;
        let mut client = AppServerClient::start(executable).await?;
        let result = client.collect_quota().await;
        client.stop().await;
        result
    }
}

#[async_trait]
impl QuotaProvider for AppServerProvider {
    async fn fetch(&self) -> AppResult<ProviderOutput> {
        match timeout(RPC_TIMEOUT, self.fetch_inner()).await {
            Ok(result) => result,
            Err(_) => Ok(ProviderOutput {
                provider: ProviderKind::AppServer,
                capabilities: CapabilityProbe {
                    account_read: CapabilityState::TemporarilyUnavailable,
                    rate_limits_read: CapabilityState::TemporarilyUnavailable,
                    usage_read: CapabilityState::TemporarilyUnavailable,
                },
                snapshot: QuotaSnapshot::unavailable(
                    ProviderKind::AppServer,
                    "Codex app-server 响应超时；未使用缓存或模拟数据替代。",
                ),
            }),
        }
    }
}

struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl AppServerClient {
    async fn start(executable: &Path) -> AppResult<Self> {
        let mut command = Command::new(executable);
        command
            .arg("app-server")
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        // The provider is a background stdio client. Do not create a visible
        // console window on Windows each time the periodic quota refresh runs.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let mut child = command.spawn().map_err(|_| {
            AppError::ProviderUnavailable("Codex app-server could not be started".into())
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::ProviderUnavailable("Codex stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::ProviderUnavailable("Codex stdout unavailable".into()))?;
        let mut client = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        let initialize = json!({
            "method": "initialize",
            "id": 0,
            "params": {
                "clientInfo": {
                    "name": "my-codex",
                    "title": "My Codex",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "optOutNotificationMethods": [
                        "item/agentMessage/delta",
                        "item/reasoning/textDelta",
                        "item/reasoning/summaryTextDelta"
                    ]
                }
            }
        });
        client.write_message(&initialize).await?;
        match client.read_response(0).await? {
            RpcOutcome::Result(_) => {}
            RpcOutcome::Error(error) => {
                return Err(AppError::ProviderUnavailable(if error.code == -32601 {
                    "Codex version does not support app-server initialization".into()
                } else {
                    "Codex app-server rejected initialization".into()
                }));
            }
        }
        client
            .write_message(&json!({ "method": "initialized", "params": {} }))
            .await?;
        Ok(client)
    }

    async fn request(&mut self, method: &str, params: Option<Value>) -> AppResult<RpcOutcome> {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let mut message = json!({ "method": method, "id": id });
        if let Some(params) = params {
            message["params"] = params;
        }
        self.write_message(&message).await?;
        self.read_response(id).await
    }

    async fn write_message(&mut self, value: &Value) -> AppResult<()> {
        let mut encoded = serde_json::to_vec(value)?;
        if encoded.len() > MAX_JSON_LINE_BYTES {
            return Err(AppError::ProviderUnavailable(
                "Codex request exceeded the local safety limit".into(),
            ));
        }
        encoded.push(b'\n');
        self.stdin.write_all(&encoded).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read_response(&mut self, expected_id: u64) -> AppResult<RpcOutcome> {
        loop {
            let line = read_bounded_line(&mut self.stdout).await?;
            if line.is_empty() {
                return Err(AppError::ProviderUnavailable(
                    "Codex app-server ended before replying".into(),
                ));
            }
            let message: RpcMessage = serde_json::from_slice(&line).map_err(|_| {
                AppError::ProviderUnavailable("Codex app-server returned invalid JSON".into())
            })?;
            if message.id != Some(expected_id) {
                continue;
            }
            if let Some(error) = message.error {
                return Ok(RpcOutcome::Error(error));
            }
            return Ok(RpcOutcome::Result(message.result.unwrap_or(Value::Null)));
        }
    }

    async fn collect_quota(&mut self) -> AppResult<ProviderOutput> {
        let mut capabilities = CapabilityProbe::default();
        let account_response = self
            .request("account/read", Some(json!({ "refreshToken": false })))
            .await?;
        let account = match account_response {
            RpcOutcome::Result(value) => match serde_json::from_value::<AccountReadResult>(value) {
                Ok(account) => {
                    capabilities.account_read = if account.account.is_some() {
                        CapabilityState::Supported
                    } else {
                        CapabilityState::AuthUnavailable
                    };
                    Some(account)
                }
                Err(_) => {
                    capabilities.account_read = CapabilityState::Incompatible;
                    None
                }
            },
            RpcOutcome::Error(error) => {
                capabilities.account_read = classify_rpc_error(&error);
                None
            }
        };

        let rate_response = self.request("account/rateLimits/read", None).await?;
        let rate_limits = match rate_response {
            RpcOutcome::Result(value) => match serde_json::from_value::<RateLimitsResult>(value) {
                Ok(result) => {
                    capabilities.rate_limits_read = CapabilityState::Supported;
                    Some(result)
                }
                Err(_) => {
                    capabilities.rate_limits_read = CapabilityState::Incompatible;
                    None
                }
            },
            RpcOutcome::Error(error) => {
                capabilities.rate_limits_read = classify_rpc_error(&error);
                None
            }
        };

        let usage = if capabilities.account_read == CapabilityState::AuthUnavailable {
            capabilities.usage_read = CapabilityState::AuthUnavailable;
            None
        } else {
            match self.request("account/usage/read", None).await? {
                RpcOutcome::Result(value) => {
                    match serde_json::from_value::<UsageReadResult>(value) {
                        Ok(result) => {
                            capabilities.usage_read = CapabilityState::Supported;
                            Some(result.into_public())
                        }
                        Err(_) => {
                            capabilities.usage_read = CapabilityState::Incompatible;
                            None
                        }
                    }
                }
                RpcOutcome::Error(error) => {
                    capabilities.usage_read = classify_rpc_error(&error);
                    None
                }
            }
        };

        let signed_out = account
            .as_ref()
            .map(|value| value.account.is_none())
            .unwrap_or(false);
        let plan_from_account = account
            .as_ref()
            .and_then(|value| value.account.as_ref())
            .and_then(|value| value.plan_type.clone());
        let selected = rate_limits
            .as_ref()
            .and_then(RateLimitsResult::select_codex_bucket);
        let plan = selected
            .and_then(|value| value.plan_type.clone())
            .or(plan_from_account);
        let primary = selected
            .and_then(|value| value.primary.as_ref())
            .map(window_from_rpc);
        let secondary = selected
            .and_then(|value| value.secondary.as_ref())
            .map(window_from_rpc);
        let reset_at = secondary
            .as_ref()
            .and_then(|value| value.resets_at)
            .or_else(|| primary.as_ref().and_then(|value| value.resets_at));
        let status = if signed_out {
            QuotaStatus::SignedOut
        } else if capabilities.rate_limits_read == CapabilityState::Supported && selected.is_some()
        {
            QuotaStatus::Available
        } else if capabilities.rate_limits_read == CapabilityState::Incompatible {
            QuotaStatus::Incompatible
        } else {
            QuotaStatus::Unavailable
        };
        let message = match status {
            QuotaStatus::Available => None,
            QuotaStatus::SignedOut => Some("Codex app-server 报告当前未登录。".into()),
            QuotaStatus::Incompatible => {
                Some("Codex app-server 返回了当前版本无法解析的额度结构。".into())
            }
            _ => Some("Codex app-server 可用，但官方额度方法当前不可用。".into()),
        };

        Ok(ProviderOutput {
            provider: ProviderKind::AppServer,
            capabilities,
            snapshot: QuotaSnapshot {
                captured_at: chrono::Utc::now(),
                provider: ProviderKind::AppServer,
                status,
                source: DataSource::Official,
                plan,
                primary,
                secondary,
                reset_at,
                tokens: None,
                official_usage: usage,
                message,
            },
        })
    }

    async fn stop(&mut self) {
        let _ = self.stdin.shutdown().await;
        let _ = self.child.start_kill();
        let _ = timeout(Duration::from_secs(1), self.child.wait()).await;
    }
}

async fn read_bounded_line<R: AsyncBufRead + Unpin>(reader: &mut R) -> AppResult<Vec<u8>> {
    let mut output = Vec::with_capacity(4096);
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            return Ok(output);
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(buffer.len(), |position| position + 1);
        if output.len().saturating_add(take) > MAX_JSON_LINE_BYTES {
            return Err(AppError::ProviderUnavailable(
                "Codex app-server response exceeded the local safety limit".into(),
            ));
        }
        output.extend_from_slice(&buffer[..take]);
        reader.consume(take);
        if newline.is_some() {
            while matches!(output.last(), Some(b'\n' | b'\r')) {
                output.pop();
            }
            return Ok(output);
        }
    }
}

#[derive(Debug, Deserialize)]
struct RpcMessage {
    id: Option<u64>,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
}

enum RpcOutcome {
    Result(Value),
    Error(RpcError),
}

fn classify_rpc_error(error: &RpcError) -> CapabilityState {
    match error.code {
        -32601 => CapabilityState::Unsupported,
        -32602 | -32600 => CapabilityState::Incompatible,
        -32001 => CapabilityState::TemporarilyUnavailable,
        _ => CapabilityState::TemporarilyUnavailable,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountReadResult {
    account: Option<AccountSummary>,
    #[allow(dead_code)]
    #[serde(default)]
    requires_openai_auth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountSummary {
    #[serde(default)]
    plan_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitsResult {
    #[serde(default)]
    rate_limits: Option<RateLimitBucket>,
    #[serde(default)]
    rate_limits_by_limit_id: HashMap<String, RateLimitBucket>,
}

impl RateLimitsResult {
    fn select_codex_bucket(&self) -> Option<&RateLimitBucket> {
        self.rate_limits_by_limit_id
            .get("codex")
            .or(self.rate_limits.as_ref())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitBucket {
    #[allow(dead_code)]
    limit_id: String,
    #[serde(default)]
    plan_type: Option<String>,
    #[serde(default)]
    primary: Option<RateLimitWindow>,
    #[serde(default)]
    secondary: Option<RateLimitWindow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitWindow {
    used_percent: f64,
    #[serde(default)]
    window_duration_mins: Option<i64>,
    #[serde(default)]
    resets_at: Option<i64>,
}

fn window_from_rpc(window: &RateLimitWindow) -> QuotaWindow {
    let used_percent = window.used_percent.clamp(0.0, 100.0);
    QuotaWindow {
        used_percent,
        remaining_percent: (100.0 - used_percent).clamp(0.0, 100.0),
        window_duration_minutes: window.window_duration_mins.filter(|value| *value > 0),
        resets_at: window.resets_at.and_then(epoch_seconds),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageReadResult {
    #[serde(default)]
    summary: Option<UsageSummaryRpc>,
    #[serde(default)]
    daily_usage_buckets: Option<Vec<DailyUsageRpc>>,
}

impl UsageReadResult {
    fn into_public(self) -> OfficialUsageSummary {
        let summary = self.summary.unwrap_or_default();
        OfficialUsageSummary {
            lifetime_tokens: summary.lifetime_tokens,
            peak_daily_tokens: summary.peak_daily_tokens,
            longest_running_turn_seconds: summary.longest_running_turn_sec,
            current_streak_days: summary.current_streak_days,
            longest_streak_days: summary.longest_streak_days,
            daily_usage: self
                .daily_usage_buckets
                .unwrap_or_default()
                .into_iter()
                .filter(|bucket| bucket.tokens >= 0 && bucket.start_date.len() <= 32)
                .map(|bucket| OfficialDailyUsage {
                    start_date: bucket.start_date,
                    tokens: bucket.tokens,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageSummaryRpc {
    lifetime_tokens: Option<i64>,
    peak_daily_tokens: Option<i64>,
    longest_running_turn_sec: Option<i64>,
    current_streak_days: Option<i64>,
    longest_streak_days: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DailyUsageRpc {
    start_date: String,
    tokens: i64,
}

fn find_codex_executable(configured: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = configured {
        if path.is_file() {
            return Some(path);
        }
    }

    let executable_names: &[&str] = if cfg!(windows) {
        // Only launch native executables directly. Batch shims would require a
        // command shell and are intentionally outside this provider's boundary.
        &["codex.exe"]
    } else {
        &["codex"]
    };
    if let Some(path) = env::var_os("PATH") {
        for directory in env::split_paths(&path) {
            #[cfg(windows)]
            if directory.starts_with(Path::new(r"C:\Program Files\WindowsApps")) {
                // The Windows App Execution Alias is visible in PATH but is
                // not directly executable from a desktop child process.
                continue;
            }
            for name in executable_names {
                let candidate = directory.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    #[cfg(windows)]
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        let bin_dir = PathBuf::from(&local_app_data)
            .join("OpenAI")
            .join("Codex")
            .join("bin");
        for name in executable_names {
            let candidate = bin_dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        // Codex Desktop stores the native CLI under a version/hash directory.
        // This is the normal installation layout on Windows.
        if let Ok(entries) = std::fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                    continue;
                }
                for name in executable_names {
                    let candidate = entry.path().join(name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        // The Microsoft Store installation is commonly visible to `where`
        // but not inherited by a Tauri process launched from a VS developer
        // shell. Resolve the installed Codex binary directly as a fallback.
        let windows_apps = Path::new(r"C:\Program Files\WindowsApps");
        if let Ok(entries) = std::fs::read_dir(windows_apps) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.starts_with("OpenAI.Codex_") {
                    continue;
                }
                for executable_name in executable_names {
                    let candidate = entry
                        .path()
                        .join("app")
                        .join("resources")
                        .join(executable_name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_quota_window_and_parses_epoch_seconds() {
        let parsed = window_from_rpc(&RateLimitWindow {
            used_percent: 125.0,
            window_duration_mins: Some(10_080),
            resets_at: Some(1_730_947_200),
        });
        assert_eq!(parsed.used_percent, 100.0);
        assert_eq!(parsed.remaining_percent, 0.0);
        assert!(parsed.resets_at.is_some());
    }

    #[test]
    fn prefers_named_codex_bucket() {
        let value = serde_json::from_value::<RateLimitsResult>(json!({
            "rateLimits": { "limitId": "fallback", "primary": null, "secondary": null },
            "rateLimitsByLimitId": {
                "codex": {
                    "limitId": "codex",
                    "primary": { "usedPercent": 25, "windowDurationMins": 15, "resetsAt": 1730947200 },
                    "secondary": null
                }
            }
        }))
        .expect("fixture should parse");
        assert_eq!(
            value.select_codex_bucket().expect("bucket").limit_id,
            "codex"
        );
    }

    #[test]
    fn usage_summary_does_not_invent_token_breakdown() {
        let usage = serde_json::from_value::<UsageReadResult>(json!({
            "summary": { "lifetimeTokens": 1234, "peakDailyTokens": null },
            "dailyUsageBuckets": [{ "startDate": "2026-06-18", "tokens": 123 }]
        }))
        .expect("fixture should parse")
        .into_public();
        assert_eq!(usage.lifetime_tokens, Some(1234));
        assert_eq!(usage.daily_usage.len(), 1);
    }
}
