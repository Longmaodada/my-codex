use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    Official,
    Local,
    Estimated,
}

impl DataSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Local => "local",
            Self::Estimated => "estimated",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBreakdown {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
}

impl TokenBreakdown {
    pub fn normalize(mut self) -> Self {
        self.input_tokens = self.input_tokens.max(0);
        self.output_tokens = self.output_tokens.max(0);
        self.cached_input_tokens = self.cached_input_tokens.clamp(0, self.input_tokens);
        self.reasoning_tokens = self.reasoning_tokens.max(0);
        self.total_tokens = self
            .total_tokens
            .max(self.input_tokens.saturating_add(self.output_tokens));
        self
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            input_tokens: self.input_tokens.saturating_add(other.input_tokens),
            output_tokens: self.output_tokens.saturating_add(other.output_tokens),
            cached_input_tokens: self
                .cached_input_tokens
                .saturating_add(other.cached_input_tokens),
            reasoning_tokens: self.reasoning_tokens.saturating_add(other.reasoning_tokens),
            total_tokens: self.total_tokens.saturating_add(other.total_tokens),
        }
        .normalize()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UsageRange {
    Today,
    SevenDays,
    #[default]
    ThirtyDays,
    NinetyDays,
    All,
}

impl UsageRange {
    pub fn cutoff(self, today: NaiveDate) -> Option<NaiveDate> {
        match self {
            Self::Today => Some(today),
            Self::SevenDays => Some(today - chrono::Duration::days(6)),
            Self::ThirtyDays => Some(today - chrono::Duration::days(29)),
            Self::NinetyDays => Some(today - chrono::Duration::days(89)),
            Self::All => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    #[serde(flatten)]
    pub tokens: TokenBreakdown,
    pub requests: i64,
    pub sessions: i64,
    pub active_seconds: i64,
    pub source: Option<DataSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub bucket: String,
    #[serde(flatten)]
    pub tokens: TokenBreakdown,
    pub requests: i64,
    pub sessions: i64,
    pub active_seconds: i64,
    pub cache_hit_ratio: f64,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRankItem {
    pub project_id: String,
    pub project_name: String,
    pub project_path: Option<String>,
    #[serde(flatten)]
    pub tokens: TokenBreakdown,
    pub requests: i64,
    pub sessions: i64,
    pub active_seconds: i64,
    pub share: f64,
    pub cache_hit_ratio: f64,
    pub first_used_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRankItem {
    pub skill_name: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub invocations: i64,
    #[serde(flatten)]
    pub tokens: TokenBreakdown,
    pub share: f64,
    pub average_tokens: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRankItem {
    pub model: String,
    #[serde(flatten)]
    pub tokens: TokenBreakdown,
    pub requests: i64,
    pub sessions: i64,
    pub share: f64,
    pub average_tokens: i64,
    pub cache_hit_ratio: f64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheAnalytics {
    pub range: UsageRange,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub uncached_input_tokens: i64,
    pub cache_hit_ratio: f64,
    pub cache_contribution_ratio: f64,
    pub estimated_saved_tokens: i64,
    pub requests: i64,
    pub trend: Vec<TrendPoint>,
    pub source: DataSource,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDay {
    pub date: NaiveDate,
    pub total_tokens: i64,
    pub requests: i64,
    pub sessions: i64,
    pub active_seconds: i64,
    pub cache_hit_ratio: f64,
    pub source: DataSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetail {
    pub summary: ProjectRankItem,
    pub trend: Vec<TrendPoint>,
    pub models: Vec<ModelRankItem>,
    pub skills: Vec<SkillRankItem>,
    pub average_session_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAnalytics {
    pub today: UsageTotals,
    pub seven_days: UsageTotals,
    pub thirty_days: UsageTotals,
    pub all_time: UsageTotals,
    pub heatmap: Vec<HeatmapDay>,
    pub projects: Vec<ProjectRankItem>,
    pub skills: Vec<SkillRankItem>,
    pub models: Vec<ModelRankItem>,
    pub cache: CacheAnalytics,
}

#[derive(Debug, Clone, Default)]
pub struct SessionAggregate {
    pub session_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub model: Option<String>,
    pub tokens: TokenBreakdown,
    pub requests: i64,
    pub active_seconds: i64,
    pub skills: Vec<SessionSkillAggregate>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionSkillAggregate {
    pub skill_name: String,
    pub invocations: i64,
    pub tokens: TokenBreakdown,
    pub last_used_at: Option<DateTime<Utc>>,
}

pub fn ratio(numerator: i64, denominator: i64) -> f64 {
    if denominator <= 0 {
        0.0
    } else {
        ((numerator.max(0) as f64 / denominator as f64) * 10_000.0).round() / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_normalization_does_not_double_count_cache_or_reasoning() {
        let tokens = TokenBreakdown {
            input_tokens: 100,
            output_tokens: 40,
            cached_input_tokens: 80,
            reasoning_tokens: 20,
            total_tokens: 0,
        }
        .normalize();

        assert_eq!(tokens.total_tokens, 140);
        assert_eq!(tokens.cached_input_tokens, 80);
    }

    #[test]
    fn ratio_is_a_percentage() {
        assert_eq!(ratio(80, 100), 80.0);
        assert_eq!(ratio(10, 0), 0.0);
    }
}
