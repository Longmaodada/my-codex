use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
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
    request_fingerprints: HashSet<String>,
    skill_evidence: HashSet<String>,
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
        let timestamp = parse_timestamp(record.timestamp.as_deref()).or_else(|| {
            string_field(&record.payload, "timestamp")
                .and_then(|value| parse_timestamp(Some(value)))
        });
        if let Some(timestamp) = timestamp {
            self.observe_timestamp(timestamp);
        }

        match record.kind.as_str() {
            "session_meta" => self.ingest_session_meta(&record.payload),
            "turn_context" => self.ingest_turn_context(&record.payload),
            "event_msg" | "response_item" => {
                self.ingest_safe_event(&record.kind, &record.payload, timestamp)
            }
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

    fn ingest_session_meta(&mut self, payload: &Value) {
        if let Some(id) = string_field(payload, "id")
            .or_else(|| string_field(payload, "session_id"))
            .and_then(|value| sanitize_identifier(Some(value)))
        {
            self.session.session_id = id;
        }
        self.set_project(string_field(payload, "cwd"));
        if let Some(model) =
            string_field(payload, "model").and_then(|value| sanitize_identifier(Some(value)))
        {
            self.session.model = Some(model);
        }
    }

    fn ingest_turn_context(&mut self, payload: &Value) {
        self.set_project(string_field(payload, "cwd"));
        if let Some(model) =
            string_field(payload, "model").and_then(|value| sanitize_identifier(Some(value)))
        {
            self.session.model = Some(model);
        }
    }

    fn ingest_safe_event(
        &mut self,
        record_kind: &str,
        payload: &Value,
        timestamp: Option<DateTime<Utc>>,
    ) {
        let event_type = string_field(payload, "type");
        if event_type == Some("token_count") {
            if let Some(info) = payload.get("info") {
                // total_token_usage is a cumulative session snapshot. Keep only
                // the greatest valid snapshot; never add snapshots together.
                let cumulative = info
                    .get("total_token_usage")
                    .or_else(|| info.get("token_usage"))
                    .and_then(parse_token_usage);
                if let Some(tokens) = cumulative {
                    self.observe_cumulative_tokens(tokens);
                }

                // last_token_usage is a per-request delta. It is used only for
                // the request count, with an event/time fingerprint so a replay
                // of the same JSONL line cannot inflate the count.
                if let Some(tokens) = info.get("last_token_usage").and_then(parse_token_usage) {
                    let event_key = format!(
                        "{}:{:?}:{:?}",
                        record_kind,
                        timestamp.map(|value| value.to_rfc3339()),
                        token_fingerprint(tokens)
                    );
                    if self.request_fingerprints.insert(event_key) {
                        self.session.requests = self.session.requests.saturating_add(1);
                    }
                }
            }
        }

        if matches!(event_type, Some("skill_invocation" | "skill_call")) {
            let names = explicit_skill_names(payload);
            let tokens = payload
                .get("token_usage")
                .or_else(|| {
                    payload
                        .get("info")
                        .and_then(|info| info.get("last_token_usage"))
                })
                .and_then(parse_token_usage)
                .unwrap_or_default();
            for skill_name in names {
                self.record_skill(
                    skill_name,
                    tokens,
                    format!("event:{:?}:{:?}:{:?}", event_type, timestamp, payload),
                    timestamp,
                );
            }
        }

        if record_kind == "response_item"
            && matches!(
                event_type,
                Some(
                    "custom_tool_call"
                        | "function_call"
                        | "custom_tool_call_output"
                        | "function_call_output"
                )
            )
        {
            let call_id = string_field(payload, "call_id")
                .or_else(|| string_field(payload, "id"))
                .unwrap_or_default();
            let tokens = payload
                .get("token_usage")
                .or_else(|| {
                    payload
                        .get("info")
                        .and_then(|info| info.get("last_token_usage"))
                })
                .and_then(parse_token_usage)
                .unwrap_or_default();
            for skill_name in tool_skill_names(payload) {
                let evidence_key = if call_id.is_empty() {
                    format!("tool:{:?}:{:?}:{:?}", event_type, timestamp, skill_name)
                } else {
                    format!("tool:{}:{}", call_id, skill_name)
                };
                self.record_skill(skill_name, tokens, evidence_key, timestamp);
            }
        }
    }

    fn observe_cumulative_tokens(&mut self, candidate: TokenBreakdown) {
        if candidate.total_tokens >= self.session.tokens.total_tokens {
            self.session.tokens = candidate;
        }
    }

    fn record_skill(
        &mut self,
        skill_name: String,
        tokens: TokenBreakdown,
        evidence_key: String,
        timestamp: Option<DateTime<Utc>>,
    ) {
        if !self.skill_evidence.insert(evidence_key) {
            return;
        }
        let entry =
            self.skills
                .entry(skill_name.clone())
                .or_insert_with(|| SessionSkillAggregate {
                    skill_name,
                    ..SessionSkillAggregate::default()
                });
        entry.invocations = entry.invocations.saturating_add(1);
        // Skill attribution is a view over usage and is never added to the
        // session total. Missing tool-level usage intentionally remains zero;
        // it must not be presented as an official token value.
        entry.tokens = entry.tokens.saturating_add(tokens);
        entry.last_used_at = timestamp.or(entry.last_used_at);
    }

    fn set_project(&mut self, cwd: Option<&str>) {
        let Some(cwd) = cwd.filter(|value| !value.is_empty() && value.len() <= MAX_PATH_LENGTH)
        else {
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
    payload: Value,
}

#[derive(Debug, Clone, Copy, Deserialize)]
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

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn parse_token_usage(value: &Value) -> Option<TokenBreakdown> {
    serde_json::from_value::<TokenUsageRecord>(value.clone())
        .ok()
        .map(|value| value.to_breakdown())
}

fn token_fingerprint(tokens: TokenBreakdown) -> [i64; 5] {
    [
        tokens.input_tokens,
        tokens.output_tokens,
        tokens.cached_input_tokens,
        tokens.reasoning_tokens,
        tokens.total_tokens,
    ]
}

fn explicit_skill_names(payload: &Value) -> Vec<String> {
    ["skill_name", "skill", "skill_id", "skill_resource"]
        .into_iter()
        .filter_map(|field| string_field(payload, field).and_then(canonical_skill_name))
        .collect()
}

fn tool_skill_names(payload: &Value) -> Vec<String> {
    let mut names = explicit_skill_names(payload);
    collect_skill_names_from_value(payload, &mut names);
    names.sort();
    names.dedup();
    names
}

fn collect_skill_names_from_value(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::String(text) => {
            for token in text.split(|character: char| {
                character.is_whitespace()
                    || matches!(
                        character,
                        '"' | '\'' | '`' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
                    )
            }) {
                if let Some(skill_name) = skill_name_from_path(token) {
                    names.push(skill_name);
                }
            }
        }
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_skill_names_from_value(value, names)),
        Value::Object(fields) => fields
            .values()
            .for_each(|value| collect_skill_names_from_value(value, names)),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn skill_name_from_path(value: &str) -> Option<String> {
    let normalized = value.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let skills_marker = "/skills/";
    let marker_start = lower.rfind(skills_marker)?;
    let skill_start = marker_start + skills_marker.len();
    let suffix = &lower[skill_start..];
    let skill_end = suffix.strip_suffix("/skill.md")?;
    if skill_end.is_empty() || skill_end.contains('/') {
        return None;
    }
    let skill = &normalized[skill_start..skill_start + skill_end.len()];
    let prefix = &lower[..marker_start];
    let name = if let Some(cache_start) = prefix.rfind("/plugins/cache/") {
        let plugin_prefix = &normalized[cache_start + "/plugins/cache/".len()..marker_start];
        let segments: Vec<_> = plugin_prefix.split('/').collect();
        if segments.len() >= 3 {
            let plugin_name = segments[segments.len() - 2];
            if plugin_name == "product-design" {
                format!("{plugin_name}:{skill}")
            } else {
                skill.to_owned()
            }
        } else {
            skill.to_owned()
        }
    } else {
        skill.to_owned()
    };
    canonical_skill_name(&name)
}

fn canonical_skill_name(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_IDENTIFIER_LENGTH {
        return None;
    }
    let canonical = match value.to_ascii_lowercase().as_str() {
        "frontend-app-builder" => "frontend-app-builder",
        "imagegen" => "imagegen",
        "product-design:image-to-code" => "product-design:image-to-code",
        "impeccable" => "impeccable",
        "vue-best-practices" => "vue-best-practices",
        "visualize" => "visualize",
        "lanshu-animated-architecture-diagram" => "lanshu-animated-architecture-diagram",
        "control-in-app-browser" => "control-in-app-browser",
        "openai-docs" => "openai-docs",
        "data-visualization" => "data-visualization",
        "visual-verdict" => "visual-verdict",
        "vue-testing-best-practices" => "vue-testing-best-practices",
        _ => value,
    };
    sanitize_skill_name(Some(canonical))
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
    (!value.is_empty()
        && value.len() <= MAX_IDENTIFIER_LENGTH
        && !value.chars().any(char::is_control))
    .then(|| value.to_owned())
}

fn sanitize_skill_name(value: Option<&str>) -> Option<String> {
    let value = sanitize_identifier(value)?;
    value
        .chars()
        .all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':' | '.')
        })
        .then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_whitelisted_metadata_from_sanitized_fixture() {
        let fixture = include_bytes!("../../tests/fixtures/session_safe.jsonl");
        let mut parser = SafeSessionParser::new("fallback".into());
        for line in fixture
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
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

    #[test]
    fn reads_skill_paths_from_tool_calls_without_adding_them_to_session_total() {
        let mut parser = SafeSessionParser::new("safe".into());
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:01Z","type":"response_item","payload":{"type":"custom_tool_call","call_id":"skill-read-1","name":"read_file","arguments":{"path":"C:\\Users\\q1384\\.codex\\skills\\frontend-app-builder\\SKILL.md"}}}"#));
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:02Z","type":"response_item","payload":{"type":"custom_tool_call_output","call_id":"skill-read-1","output":[{"type":"input_text","text":"C:\\Users\\q1384\\.codex\\skills\\frontend-app-builder\\SKILL.md"}]}}"#));
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:03Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"output_tokens":20,"total_tokens":120},"last_token_usage":{"input_tokens":100,"output_tokens":20,"total_tokens":120}}}}"#));
        let session = parser.finish();
        assert_eq!(session.tokens.total_tokens, 120);
        assert_eq!(session.skills.len(), 1);
        assert_eq!(session.skills[0].skill_name, "frontend-app-builder");
        assert_eq!(session.skills[0].invocations, 1);
        assert_eq!(session.skills[0].tokens.total_tokens, 0);
    }

    #[test]
    fn ignores_colon_delimited_non_skill_values_in_tool_payloads() {
        let mut parser = SafeSessionParser::new("safe".into());
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:01Z","type":"response_item","payload":{"type":"function_call","call_id":"not-a-skill","arguments":"{\"selector\":\"button:hover\",\"filter\":\"exact:true\"}"}}"#));
        assert!(parser.finish().skills.is_empty());
    }

    #[test]
    fn uses_final_monotonic_cumulative_snapshot_and_deduplicates_requests() {
        let mut parser = SafeSessionParser::new("safe".into());
        let first = br#"{"timestamp":"2026-08-12T00:00:01Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":80,"output_tokens":20,"total_tokens":100},"last_token_usage":{"input_tokens":80,"output_tokens":20,"total_tokens":100}}}}"#;
        assert!(parser.ingest_line(first));
        assert!(parser.ingest_line(first));
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:02Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":60,"output_tokens":20,"total_tokens":80},"last_token_usage":{"input_tokens":60,"output_tokens":20,"total_tokens":80}}}}"#));
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:03Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":120,"output_tokens":30,"total_tokens":150},"last_token_usage":{"input_tokens":40,"output_tokens":10,"total_tokens":50}}}}"#));
        let session = parser.finish();
        assert_eq!(session.tokens.total_tokens, 150);
        assert_eq!(session.requests, 3);
    }

    #[test]
    fn canonicalizes_plugin_skill_paths() {
        let mut parser = SafeSessionParser::new("safe".into());
        assert!(parser.ingest_line(br#"{"timestamp":"2026-08-12T00:00:01Z","type":"response_item","payload":{"type":"function_call","call_id":"skill-read-2","arguments":"{\"path\":\"C:/Users/q1384/.codex/plugins/cache/openai-curated-remote/product-design/0.1.52/skills/image-to-code/SKILL.md\"}"}}"#));
        let session = parser.finish();
        assert_eq!(session.skills[0].skill_name, "product-design:image-to-code");
    }
}
