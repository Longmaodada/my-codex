use chrono::NaiveDate;
use rusqlite::params;

use super::{cutoff_string, parse_datetime, Database};
use crate::{
    analytics::{
        ratio, CacheAnalytics, DashboardAnalytics, DataSource, HeatmapDay, ModelRankItem,
        ProjectDetail, ProjectRankItem, SkillRankItem, TokenBreakdown, TrendPoint, UsageRange,
        UsageTotals,
    },
    error::{AppError, AppResult},
};

impl Database {
    pub fn usage_totals(&self, range: UsageRange) -> AppResult<UsageTotals> {
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0),
                        COALESCE(SUM(cached_input_tokens), 0),
                        COALESCE(SUM(reasoning_tokens), 0),
                        COALESCE(SUM(total_tokens), 0),
                        COALESCE(SUM(requests), 0),
                        COALESCE(SUM(sessions), 0),
                        COALESCE(SUM(active_seconds), 0)
                 FROM daily_usage WHERE (?1 IS NULL OR date >= ?1)",
                params![cutoff],
                |row| {
                    let sessions: i64 = row.get(6)?;
                    Ok(UsageTotals {
                        tokens: TokenBreakdown {
                            input_tokens: row.get(0)?,
                            output_tokens: row.get(1)?,
                            cached_input_tokens: row.get(2)?,
                            reasoning_tokens: row.get(3)?,
                            total_tokens: row.get(4)?,
                        }
                        .normalize(),
                        requests: row.get(5)?,
                        sessions,
                        active_seconds: row.get(7)?,
                        source: (sessions > 0).then_some(DataSource::Local),
                    })
                },
            )
            .map_err(AppError::from)
    }

    pub fn usage_trend(&self, range: UsageRange) -> AppResult<Vec<TrendPoint>> {
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT date, input_tokens, output_tokens, cached_input_tokens,
                    reasoning_tokens, total_tokens, requests, sessions, active_seconds
             FROM daily_usage WHERE (?1 IS NULL OR date >= ?1)
             ORDER BY date ASC",
        )?;
        let rows = statement.query_map(params![cutoff], |row| {
            let input: i64 = row.get(1)?;
            let cached: i64 = row.get(3)?;
            Ok(TrendPoint {
                bucket: row.get(0)?,
                tokens: TokenBreakdown {
                    input_tokens: input,
                    output_tokens: row.get(2)?,
                    cached_input_tokens: cached,
                    reasoning_tokens: row.get(4)?,
                    total_tokens: row.get(5)?,
                }
                .normalize(),
                requests: row.get(6)?,
                sessions: row.get(7)?,
                active_seconds: row.get(8)?,
                cache_hit_ratio: ratio(cached, input),
                source: DataSource::Local,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn heatmap(&self, days: u16) -> AppResult<Vec<HeatmapDay>> {
        let bounded_days = days.clamp(1, 366);
        let cutoff = (chrono::Local::now().date_naive()
            - chrono::Duration::days(i64::from(bounded_days - 1)))
        .to_string();
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT date, input_tokens, cached_input_tokens, total_tokens,
                    requests, sessions, active_seconds
             FROM daily_usage WHERE date >= ?1 ORDER BY date ASC",
        )?;
        let rows = statement.query_map(params![cutoff], |row| {
            let date_text: String = row.get(0)?;
            let date = NaiveDate::parse_from_str(&date_text, "%Y-%m-%d").map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            let input: i64 = row.get(1)?;
            let cached: i64 = row.get(2)?;
            Ok(HeatmapDay {
                date,
                total_tokens: row.get(3)?,
                requests: row.get(4)?,
                sessions: row.get(5)?,
                active_seconds: row.get(6)?,
                cache_hit_ratio: ratio(cached, input),
                source: DataSource::Local,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn projects(&self, range: UsageRange, limit: u16) -> AppResult<Vec<ProjectRankItem>> {
        let cutoff = cutoff_string(range);
        let total = self.usage_totals(range)?.tokens.total_tokens;
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT project_id, MAX(project_name), MAX(project_path),
                    SUM(input_tokens), SUM(output_tokens), SUM(cached_input_tokens),
                    SUM(reasoning_tokens), SUM(total_tokens), SUM(requests),
                    SUM(sessions), SUM(active_seconds), MIN(first_used_at), MAX(last_used_at)
             FROM project_usage WHERE (?1 IS NULL OR date >= ?1)
             GROUP BY project_id ORDER BY SUM(total_tokens) DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![cutoff, i64::from(limit.clamp(1, 500))], |row| {
            let input: i64 = row.get(3)?;
            let cached: i64 = row.get(5)?;
            let item_total: i64 = row.get(7)?;
            Ok(ProjectRankItem {
                project_id: row.get(0)?,
                project_name: row.get(1)?,
                project_path: row.get(2)?,
                tokens: TokenBreakdown {
                    input_tokens: input,
                    output_tokens: row.get(4)?,
                    cached_input_tokens: cached,
                    reasoning_tokens: row.get(6)?,
                    total_tokens: item_total,
                }
                .normalize(),
                requests: row.get(8)?,
                sessions: row.get(9)?,
                active_seconds: row.get(10)?,
                share: ratio(item_total, total),
                cache_hit_ratio: ratio(cached, input),
                first_used_at: parse_datetime(row.get(11)?),
                last_used_at: parse_datetime(row.get(12)?),
                source: DataSource::Local,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn skills(&self, range: UsageRange, limit: u16) -> AppResult<Vec<SkillRankItem>> {
        self.skills_for_project(range, None, limit)
    }

    fn skills_for_project(
        &self,
        range: UsageRange,
        project_id: Option<&str>,
        limit: u16,
    ) -> AppResult<Vec<SkillRankItem>> {
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        let total: i64 = connection.query_row(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM skill_usage
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR project_id = ?2)",
            params![cutoff, project_id],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "SELECT skill_name, SUM(invocations), SUM(input_tokens), SUM(output_tokens),
                    SUM(cached_input_tokens), SUM(reasoning_tokens), SUM(total_tokens),
                    MAX(last_used_at),
                    CASE WHEN COUNT(DISTINCT project_id) = 1 THEN MAX(NULLIF(project_id, '')) END,
                    CASE WHEN COUNT(DISTINCT project_id) = 1 THEN MAX(project_name) END
             FROM skill_usage
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR project_id = ?2)
             GROUP BY skill_name ORDER BY SUM(total_tokens) DESC, SUM(invocations) DESC LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![cutoff, project_id, i64::from(limit.clamp(1, 500))],
            |row| {
                let invocations: i64 = row.get(1)?;
                let item_total: i64 = row.get(6)?;
                Ok(SkillRankItem {
                    skill_name: row.get(0)?,
                    project_id: row.get(8)?,
                    project_name: row.get(9)?,
                    invocations,
                    tokens: TokenBreakdown {
                        input_tokens: row.get(2)?,
                        output_tokens: row.get(3)?,
                        cached_input_tokens: row.get(4)?,
                        reasoning_tokens: row.get(5)?,
                        total_tokens: item_total,
                    }
                    .normalize(),
                    share: ratio(item_total, total),
                    average_tokens: if invocations > 0 {
                        item_total / invocations
                    } else {
                        0
                    },
                    last_used_at: parse_datetime(row.get(7)?),
                    source: DataSource::Local,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn models(&self, range: UsageRange, limit: u16) -> AppResult<Vec<ModelRankItem>> {
        self.models_for_project(range, None, limit)
    }

    fn models_for_project(
        &self,
        range: UsageRange,
        project_id: Option<&str>,
        limit: u16,
    ) -> AppResult<Vec<ModelRankItem>> {
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        let total: i64 = connection.query_row(
            "SELECT COALESCE(SUM(total_tokens), 0) FROM model_usage
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR project_id = ?2)",
            params![cutoff, project_id],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "SELECT model, SUM(input_tokens), SUM(output_tokens),
                    SUM(cached_input_tokens), SUM(reasoning_tokens), SUM(total_tokens),
                    SUM(requests), SUM(sessions), MAX(last_used_at)
             FROM model_usage
             WHERE (?1 IS NULL OR date >= ?1) AND (?2 IS NULL OR project_id = ?2)
             GROUP BY model ORDER BY SUM(total_tokens) DESC LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![cutoff, project_id, i64::from(limit.clamp(1, 500))],
            |row| {
                let input: i64 = row.get(1)?;
                let cached: i64 = row.get(3)?;
                let item_total: i64 = row.get(5)?;
                let requests: i64 = row.get(6)?;
                Ok(ModelRankItem {
                    model: row.get(0)?,
                    tokens: TokenBreakdown {
                        input_tokens: input,
                        output_tokens: row.get(2)?,
                        cached_input_tokens: cached,
                        reasoning_tokens: row.get(4)?,
                        total_tokens: item_total,
                    }
                    .normalize(),
                    requests,
                    sessions: row.get(7)?,
                    share: ratio(item_total, total),
                    average_tokens: if requests > 0 { item_total / requests } else { 0 },
                    cache_hit_ratio: ratio(cached, input),
                    last_used_at: parse_datetime(row.get(8)?),
                    source: DataSource::Local,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn cache_analytics(&self, range: UsageRange) -> AppResult<CacheAnalytics> {
        let totals = self.usage_totals(range)?;
        let input = totals.tokens.input_tokens;
        let cached = totals.tokens.cached_input_tokens.min(input).max(0);
        Ok(CacheAnalytics {
            range,
            input_tokens: input,
            cached_input_tokens: cached,
            uncached_input_tokens: input.saturating_sub(cached),
            cache_hit_ratio: ratio(cached, input),
            cache_contribution_ratio: ratio(cached, totals.tokens.total_tokens),
            estimated_saved_tokens: cached,
            requests: totals.requests,
            trend: self.usage_trend(range)?,
            source: DataSource::Local,
            disclaimer: "本地统计：缓存 Token 来自本机 Codex 会话事件，并非官方账户账单；节省量为等量 Token 估计。".into(),
        })
    }

    pub fn project_detail(
        &self,
        project_id: &str,
        range: UsageRange,
    ) -> AppResult<Option<ProjectDetail>> {
        let summary = self
            .projects(range, 500)?
            .into_iter()
            .find(|project| project.project_id == project_id);
        let Some(summary) = summary else {
            return Ok(None);
        };
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT date, SUM(input_tokens), SUM(output_tokens),
                    SUM(cached_input_tokens), SUM(reasoning_tokens), SUM(total_tokens),
                    SUM(requests), SUM(sessions), SUM(active_seconds)
             FROM project_usage WHERE project_id = ?1 AND (?2 IS NULL OR date >= ?2)
             GROUP BY date ORDER BY date ASC",
        )?;
        let rows = statement.query_map(params![project_id, cutoff], |row| {
            let input: i64 = row.get(1)?;
            let cached: i64 = row.get(3)?;
            Ok(TrendPoint {
                bucket: row.get(0)?,
                tokens: TokenBreakdown {
                    input_tokens: input,
                    output_tokens: row.get(2)?,
                    cached_input_tokens: cached,
                    reasoning_tokens: row.get(4)?,
                    total_tokens: row.get(5)?,
                }
                .normalize(),
                requests: row.get(6)?,
                sessions: row.get(7)?,
                active_seconds: row.get(8)?,
                cache_hit_ratio: ratio(cached, input),
                source: DataSource::Local,
            })
        })?;
        let trend = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?;
        drop(statement);
        drop(connection);
        let models = self.models_for_project(range, Some(project_id), 100)?;
        let skills = self.skills_for_project(range, Some(project_id), 100)?;
        let average_session_tokens = if summary.sessions > 0 {
            summary.tokens.total_tokens / summary.sessions
        } else {
            0
        };
        Ok(Some(ProjectDetail {
            summary,
            trend,
            models,
            skills,
            average_session_tokens,
        }))
    }

    pub fn dashboard_analytics(&self) -> AppResult<DashboardAnalytics> {
        Ok(DashboardAnalytics {
            today: self.usage_totals(UsageRange::Today)?,
            seven_days: self.usage_totals(UsageRange::SevenDays)?,
            thirty_days: self.usage_totals(UsageRange::ThirtyDays)?,
            all_time: self.usage_totals(UsageRange::All)?,
            heatmap: self.heatmap(90)?,
            projects: self.projects(UsageRange::ThirtyDays, 20)?,
            skills: self.skills(UsageRange::ThirtyDays, 20)?,
            models: self.models(UsageRange::ThirtyDays, 20)?,
            cache: self.cache_analytics(UsageRange::NinetyDays)?,
        })
    }
}
