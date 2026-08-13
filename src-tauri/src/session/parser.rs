use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::analytics::{SessionAggregate, SessionSkillAggregate, TokenBreakdown};

const MAX_IDENTIFIER_LENGTH: usize = 256;
const MAX_PATH_LENGTH: usize = 32_768;
const ACTIVE_GAP_CAP_SECONDS: i64 = 300;

#[derive(Debug, Default)]
pub struct SafeSessionParser {
    session: SessionAggregate,
    last_event_at: Option<DateTime<Utc>>,
    skills: HashMap<String, SessionSkillAggregate>,
}

impl SafeSessionParser {
    pub fn new(fallback_session_id: String) -> Self {
        Self {
            session: SessionAggregate {
                session_id: fallback_session_id,
                ..SessionAggregate::default()
            },
            ..Self::default()
        }
    }

    pub fn ingest_line(&mut self, line: &[u8]) -> bool {
        let Ok(record) = serde_json::from_slice::<SafeRecord>(line) else {
            return false;
        };
        let timestamp = parse_timestamp(record.timestamp.as_deref())
            .or_else(|| parse_timestamp(record.payload.timestamp.as_deref()));
        if let Some(timestamp) = timestamp {
            self.observe_timestamp(timestamp);
        }

        match record.kind.as_str() {
            "session_meta" => self.ingest_session_meta(record.payload),
            "turn_context" => self.ingest_turn_context(record.payload),
            "event_msg" | "response_item" => self.ingest_safe_event(record.payload, timestamp),
            _ => {}
        }
        true
    }

    pub fn finish(mut self) -> SessionAggregate {
        let mut skills: Vec<_> = self.skills.into_values().collect();
        skills.sort_by(|left, right| left.skill_name.cmp(&right.skill_name));
        self.session.skills = skills;
        self.session
    }

    fn observe_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.session.started_at = Some(
            self.session
                .started_at
                .map_or(timestamp, |current| current.min(timestamp)),
        );
        self.session.ended_at = Some(
            self.session
                .ended_at
                .map_or(timestamp, |current| current.max(timestamp)),
        );
        if let Some(previous) = self.last_event_at {
            let gap = timestamp.signed_duration_since(previous).num_seconds();
            if gap > 0 {
                self.session.active_seconds = self
                    .session
                    .active_seconds
                    .saturating_add(gap.min(ACTIVE_GAP_CAP_SECONDS));
            }
        }
        self.last_event_at = Some(timestamp);
    }

    fn ingest_session_meta(&mut self, payload: SafePayload) {
        if let Some(id) = sanitize_identifier(payload.id.as_deref()) {
            self.session.session_id = id;
        }
        self.set_project(payload.cwd.as_deref());
        if let Some(model) = sanitize_identifier(payload.model.as_deref()) {
            self.session.model = Some(model);
        }
    }

    fn ingest_turn_context(&mut self, payload: SafePayload) {
        self.set_project(payload.cwd.as_deref());
        if let Some(model) = sanitize_identifier(payload.model.as_deref()) {
            self.session.model = Some(model);
        }
    }

    fn ingest_safe_event(&mut self, payload: SafePayload, timestamp: Option<DateTime<Utc>>) {
        if payload.event_type.as_deref() == Some("token_count") {
            if let Some(info) = payload.info.as_ref() {
                if let Some(tokens) = info.total_token_usage.as_ref().or(info.token_usage.as_ref()) {
                    self.session.tokens = tokens.to_breakdown();
                }
                if info.last_token_usage.is_some() {
                    self.session.requests = self.session.requests.saturating_add(1);
                }
            }
        }

        if matches!(
            payload.event_type.as_deref(),
            Some("skill_invocation" | "skill_call")
        ) {
            if let Some(skill_name) = sanitize_skill_name(
                payload
                    .skill_name
                    .as_deref()
                    .or(payload.name.as_deref()),
            ) {
                let tokens = payload
                    .token_usage
                    .as_ref()
                    .or_else(|| payload.info.as_ref().and_then(|info| info.last_token_usage.as_ref()))
                    .map(TokenUsageRecord::to_breakdown)
                    .unwrap_or_default();
                let entry = self.skills.entry(skill_name.clone()).or_insert_with(|| {
                    SessionSkillAggregate {
                        skill_name,
                        ..SessionSkillAggregate::default()
                    }
                });
                entry.invocations = entry.invocations.saturating_add(1);
                entry.tokens = entry.tokens.saturating_add(tokens);
                entry.last_used_at = timestamp.or(entry.last_used_at);
            }
        }
    }

    fn set_project(&mut self, cwd: Option<&str>) {
        let Some(cwd) = cwd.filter(|value| !value.is_empty() && value.len() <= MAX_PATH_LENGTH) else {
            return;
        };
        let normalized = if cfg!(windows) {
            cwd.replace('/', "\\").to_lowercase()
        } else {
            cwd.to_owned()
        };
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        self.session.project_id = Some(format!("{:x}", hasher.finalize()));
        self.session.project_path = Some(cwd.to_owned());
        self.session.project_name = std::path::Path::new(cwd)
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(|value| sanitize_identifier(Some(value)))
            .or_else(|| Some("未命名项目".into()));
    }
}

#[derive(Debug, Deserialize)]
struct SafeRecord {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    payload: SafePayload,
}

#[derive(Debug, Default, Deserialize)]
struct SafePayload {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default, rename = "type")]
    event_type: Option<String>,
    #[serde(default)]
    info: Option<TokenInfoRecord>,
    #[serde(default)]
    token_usage: Option<TokenUsageRecord>,
    #[serde(default)]
    skill_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenInfoRecord {
    #[serde(default)]
    total_token_usage: Option<TokenUsageRecord>,
    #[serde(default)]
    last_token_usage: Option<TokenUsageRecord>,
    #[serde(default)]
    token_usage: Option<TokenUsageRecord>,
}

#[derive(Debug, Deserialize)]
struct TokenUsageRecord {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    cached_input_tokens: i64,
    #[serde(default, alias = "reasoning_tokens")]
    reasoning_output_tokens: i64,
    #[serde(default)]
    total_tokens: i64,
}

impl TokenUsageRecord {
    fn to_breakdown(&self) -> TokenBreakdown {
        TokenBreakdown {
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cached_input_tokens: self.cached_input_tokens,
            reasoning_tokens: self.reasoning_output_tokens,
            total_tokens: self.total_tokens,
        }
        .normalize()
    }
}

fn parse_timestamp(value: Option<&str>) -> Option<DateTime<Utc>> {
    value.and_then(|value| {
        DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    })
}

fn sanitize_identifier(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    (!value.is_empty() && value.len() <= MAX_IDENTIFIER_LENGTH && !value.chars().any(char::is_control))
        .then(|| value.to_owned())
}

fn sanitize_skill_name(value: Option<&str>) -> Option<String> {
    let value = sanitize_identifier(value)?;
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':' | '.'))
        .then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_whitelisted_metadata_from_sanitized_fixture() {
        let fixture = include_bytes!("../../tests/fixtures/session_safe.jsonl");
        let mut parser = SafeSessionParser::new("fallback".into());
        for line in fixture.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
            assert!(parser.ingest_line(line));
        }
        let session = parser.finish();
        assert_eq!(session.session_id, "safe-session-001");
        assert_eq!(session.model.as_deref(), Some("gpt-test"));
        assert_eq!(session.tokens.total_tokens, 150);
        assert_eq!(session.requests, 1);
        assert_eq!(session.skills.len(), 1);
        assert_eq!(session.skills[0].skill_name, "browser");
    }

    #[test]
    fn ignores_unstructured_skill_words() {
        let mut parser = SafeSessionParser::new("safe".into());
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:00Z","type":"event_msg","payload":{"type":"user_message","message":"redacted"}}"#));
        assert!(parser.finish().skills.is_empty());
    }
}
