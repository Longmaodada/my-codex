use chrono::NaiveDate;
use rusqlite::params;

use super::{cutoff_string, parse_datetime, Database};
use crate::{
    analytics::{
        ratio, CacheAnalytics, DashboardAnalytics, DataSource, HeatmapDay, ModelRankItem,
        ProjectDetail, ProjectRankItem, SkillRankItem, TaskSkillItem, TaskUsage, TokenBreakdown,
        TrendPoint, UsageRange, UsageTotals,
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

    pub fn tasks(&self, range: UsageRange, limit: u16) -> AppResult<Vec<TaskUsage>> {
        let cutoff = cutoff_string(range);
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT session_id, started_at, ended_at, project_name, model,
                    input_tokens, output_tokens, cached_input_tokens, reasoning_tokens,
                    total_tokens, requests, active_seconds
             FROM session_usage
             WHERE (?1 IS NULL OR date(started_at, 'localtime') >= ?1)
             ORDER BY started_at DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![cutoff, i64::from(limit.clamp(1, 500))], |row| {
            let input: i64 = row.get(5)?;
            let cached: i64 = row.get(7)?;
            Ok(TaskUsage {
                session_id: row.get(0)?,
                started_at: parse_datetime(Some(row.get(1)?)),
                ended_at: parse_datetime(row.get(2)?),
                project_name: row.get(3)?,
                model: row.get(4)?,
                tokens: TokenBreakdown {
                    input_tokens: input,
                    output_tokens: row.get(6)?,
                    cached_input_tokens: cached,
                    reasoning_tokens: row.get(8)?,
                    total_tokens: row.get(9)?,
                }
                .normalize(),
                requests: row.get(10)?,
                active_seconds: row.get(11)?,
                skills: Vec::new(),
                source: DataSource::Local,
            })
        })?;
        let mut tasks = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?;
        drop(statement);

        for task in &mut tasks {
            let mut skill_statement = connection.prepare(
                "SELECT skill_name, invocations FROM session_skills
                 WHERE session_id = ?1 ORDER BY last_used_at DESC, skill_name ASC",
            )?;
            let skills = skill_statement
                .query_map(params![task.session_id], |row| {
                    Ok(TaskSkillItem {
                        skill_name: row.get(0)?,
                        invocations: row.get(1)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(AppError::from)?;
            task.skills = skills;
        }
        Ok(tasks)
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
            "SELECT COALESCE(SUM(s.total_tokens), 0)
             FROM session_usage s
             WHERE EXISTS (
                 SELECT 1 FROM session_skills sk WHERE sk.session_id = s.session_id
             )
               AND (?1 IS NULL OR date(s.started_at, 'localtime') >= ?1)
               AND (?2 IS NULL OR s.project_id = ?2)",
            params![cutoff, project_id],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "WITH session_weights AS (
                 SELECT session_id, SUM(invocations) AS total_invocations
                 FROM session_skills
                 GROUP BY session_id
             )
             SELECT sk.skill_name,
                    SUM(sk.invocations),
                    SUM(CAST(ROUND(CAST(s.input_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)),
                    SUM(CAST(ROUND(CAST(s.output_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)),
                    SUM(CAST(ROUND(CAST(s.cached_input_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)),
                    SUM(CAST(ROUND(CAST(s.reasoning_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)),
                    SUM(CAST(ROUND(CAST(s.total_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)),
                    MAX(COALESCE(sk.last_used_at, s.ended_at, s.started_at)),
                    CASE WHEN COUNT(DISTINCT COALESCE(s.project_id, '')) = 1 THEN MAX(NULLIF(s.project_id, '')) END,
                    CASE WHEN COUNT(DISTINCT COALESCE(s.project_id, '')) = 1 THEN MAX(s.project_name) END,
                    CASE WHEN SUM(sk.invocations) > 0 THEN
                        SUM(CAST(sk.invocations AS REAL) * sk.invocations / sw.total_invocations) / SUM(sk.invocations)
                    END
             FROM session_skills sk
             JOIN session_usage s ON s.session_id = sk.session_id
             JOIN session_weights sw ON sw.session_id = sk.session_id
             WHERE sw.total_invocations > 0
               AND (?1 IS NULL OR date(s.started_at, 'localtime') >= ?1)
               AND (?2 IS NULL OR s.project_id = ?2)
             GROUP BY sk.skill_name
             ORDER BY SUM(CAST(ROUND(CAST(s.total_tokens AS REAL) * sk.invocations / sw.total_invocations) AS INTEGER)) DESC,
                      SUM(sk.invocations) DESC,
                      sk.skill_name ASC
             LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![cutoff, project_id, i64::from(limit.clamp(1, 500))],
            |row| {
                let invocations: i64 = row.get(1)?;
                let input: i64 = row.get(2)?;
                let cached: i64 = row.get(4)?;
                let item_total: i64 = row.get(6)?;
                let attribution_confidence: Option<f64> = row
                    .get::<_, Option<f64>>(10)?
                    .map(|value| value.clamp(0.0, 1.0));
                Ok(SkillRankItem {
                    skill_name: row.get(0)?,
                    project_id: row.get(8)?,
                    project_name: row.get(9)?,
                    invocations,
                    tokens: TokenBreakdown {
                        input_tokens: input,
                        output_tokens: row.get(3)?,
                        cached_input_tokens: cached,
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
                    cache_hit_ratio: (input > 0).then(|| ratio(cached, input)),
                    attribution_confidence,
                    last_used_at: parse_datetime(row.get(7)?),
                    // Token fields are allocated from the containing local
                    // session by recorded Skill invocation weight. They are
                    // useful local estimates, not tool-level billing data.
                    source: DataSource::Estimated,
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
                    average_tokens: if requests > 0 {
                        item_total / requests
                    } else {
                        0
                    },
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
            tasks: self.tasks(UsageRange::Today, 100)?,
            skills: self.skills(UsageRange::ThirtyDays, 20)?,
            models: self.models(UsageRange::ThirtyDays, 20)?,
            cache: self.cache_analytics(UsageRange::NinetyDays)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::Database;
    use crate::analytics::{SessionAggregate, SessionSkillAggregate, TokenBreakdown, UsageRange};

    #[test]
    fn skill_ranking_allocates_session_usage_by_recorded_invocations() {
        let database = Database::open_in_memory().expect("database");
        let now = Utc::now();
        let session = SessionAggregate {
            session_id: "skill-ranking-session".into(),
            started_at: Some(now),
            ended_at: Some(now),
            tokens: TokenBreakdown {
                input_tokens: 120,
                output_tokens: 80,
                cached_input_tokens: 60,
                reasoning_tokens: 10,
                total_tokens: 200,
            },
            requests: 1,
            skills: vec![
                SessionSkillAggregate {
                    skill_name: "browser".into(),
                    invocations: 1,
                    last_used_at: Some(now),
                    ..SessionSkillAggregate::default()
                },
                SessionSkillAggregate {
                    skill_name: "github".into(),
                    invocations: 1,
                    last_used_at: Some(now),
                    ..SessionSkillAggregate::default()
                },
            ],
            ..SessionAggregate::default()
        };

        database
            .upsert_session(&session, "skill-ranking-source", 1, 1)
            .expect("store session");
        database.rebuild_analytics().expect("rebuild analytics");

        let skills = database
            .skills(UsageRange::All, 20)
            .expect("load skill ranking");
        let browser = skills
            .iter()
            .find(|skill| skill.skill_name == "browser")
            .expect("browser skill");

        assert_eq!(browser.tokens.input_tokens, 60);
        assert_eq!(browser.tokens.output_tokens, 40);
        assert_eq!(browser.tokens.cached_input_tokens, 30);
        assert_eq!(browser.tokens.total_tokens, 100);
        assert_eq!(browser.cache_hit_ratio, Some(50.0));
        assert_eq!(browser.attribution_confidence, Some(0.5));
    }
}
