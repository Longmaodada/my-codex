mod migrations;
mod queries;

use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::{
    analytics::{SessionAggregate, UsageRange},
    error::{AppError, AppResult},
    quota::QuotaSnapshot,
    settings::AppSettings,
};

use migrations::MIGRATIONS;

pub const SESSION_PARSER_VERSION: i64 = 3;

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> AppResult<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> AppResult<Self> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;",
        )?;
        let database = Self {
            connection: Mutex::new(connection),
        };
        database.migrate()?;
        Ok(database)
    }

    fn connection(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| AppError::StatePoisoned)
    }

    fn migrate(&self) -> AppResult<()> {
        let connection = self.connection()?;
        let current: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        for (index, migration) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            connection.execute_batch(migration)?;
            let version = index + 1;
            connection.pragma_update(None, "user_version", version as i64)?;
        }
        Ok(())
    }

    pub fn load_settings(&self) -> AppResult<AppSettings> {
        let connection = self.connection()?;
        let json: Option<String> = connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'app'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match json {
            Some(json) => serde_json::from_str::<AppSettings>(&json)?.validate(),
            None => Ok(AppSettings::default()),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        let json = serde_json::to_string(settings)?;
        self.connection()?.execute(
            "INSERT INTO settings(key, value_json, updated_at) VALUES('app', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
             updated_at = excluded.updated_at",
            params![json, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn insert_quota_snapshot(&self, quota: &QuotaSnapshot) -> AppResult<()> {
        // Official app-server quota responses do not contain a token-category
        // breakdown. Keep those columns NULL instead of manufacturing zeros;
        // local session analytics and explicit mock data are the only sources
        // that may populate them.
        let tokens = quota.tokens;
        self.connection()?.execute(
            "INSERT INTO quota_snapshots(
                captured_at, provider, status, source, plan,
                primary_used_percent, primary_remaining_percent,
                primary_window_minutes, primary_resets_at,
                secondary_used_percent, secondary_remaining_percent,
                secondary_window_minutes, secondary_resets_at, reset_at,
                input_tokens, output_tokens, cached_input_tokens, reasoning_tokens,
                total_tokens
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                quota.captured_at.to_rfc3339(),
                quota.provider.as_str(),
                quota.status.as_str(),
                quota.source.as_str(),
                quota.plan,
                quota.primary.as_ref().map(|window| window.used_percent),
                quota.primary.as_ref().map(|window| window.remaining_percent),
                quota.primary.as_ref().and_then(|window| window.window_duration_minutes),
                quota.primary.as_ref().and_then(|window| window.resets_at).map(|value| value.to_rfc3339()),
                quota.secondary.as_ref().map(|window| window.used_percent),
                quota.secondary.as_ref().map(|window| window.remaining_percent),
                quota.secondary.as_ref().and_then(|window| window.window_duration_minutes),
                quota.secondary.as_ref().and_then(|window| window.resets_at).map(|value| value.to_rfc3339()),
                quota.reset_at.map(|value| value.to_rfc3339()),
                tokens.map(|value| value.input_tokens),
                tokens.map(|value| value.output_tokens),
                tokens.map(|value| value.cached_input_tokens),
                tokens.map(|value| value.reasoning_tokens),
                tokens.map(|value| value.total_tokens),
            ],
        )?;
        Ok(())
    }

    pub fn latest_quota_snapshot(&self) -> AppResult<Option<QuotaSnapshot>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT captured_at, provider, status, source, plan,
                        primary_used_percent, primary_remaining_percent,
                        primary_window_minutes, primary_resets_at,
                        secondary_used_percent, secondary_remaining_percent,
                        secondary_window_minutes, secondary_resets_at, reset_at,
                        input_tokens, output_tokens, cached_input_tokens,
                        reasoning_tokens, total_tokens
                 FROM quota_snapshots
                 WHERE status = 'available' AND provider = 'app_server' AND source = 'official'
                 ORDER BY captured_at DESC LIMIT 1",
                [],
                |row| QuotaSnapshot::from_database_row(row),
            )
            .optional()
            .map_err(AppError::from)
    }

    pub fn is_ingest_file_current(
        &self,
        source_hash: &str,
        byte_size: i64,
        modified_ms: i64,
    ) -> AppResult<bool> {
        let connection = self.connection()?;
        let current: Option<i64> = connection
            .query_row(
                "SELECT 1 FROM ingest_files
                 WHERE source_hash = ?1 AND byte_size = ?2 AND modified_ms = ?3
                   AND parser_version = ?4",
                params![source_hash, byte_size, modified_ms, SESSION_PARSER_VERSION],
                |row| row.get(0),
            )
            .optional()?;
        Ok(current.is_some())
    }

    pub fn upsert_session(
        &self,
        session: &SessionAggregate,
        source_hash: &str,
        byte_size: i64,
        modified_ms: i64,
    ) -> AppResult<()> {
        let started_at = session
            .started_at
            .ok_or_else(|| AppError::Other("session has no safe timestamp metadata".into()))?;
        let tokens = session.tokens.normalize();
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;

        transaction.execute(
            "INSERT INTO session_usage(
                session_id, started_at, ended_at, project_id, project_name, project_path,
                model, input_tokens, output_tokens, cached_input_tokens, reasoning_tokens,
                total_tokens, requests, active_seconds, source, updated_at
             ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'local', ?15)
             ON CONFLICT(session_id) DO UPDATE SET
                started_at = excluded.started_at, ended_at = excluded.ended_at,
                project_id = excluded.project_id, project_name = excluded.project_name,
                project_path = excluded.project_path, model = excluded.model,
                input_tokens = excluded.input_tokens, output_tokens = excluded.output_tokens,
                cached_input_tokens = excluded.cached_input_tokens,
                reasoning_tokens = excluded.reasoning_tokens,
                total_tokens = excluded.total_tokens, requests = excluded.requests,
                active_seconds = excluded.active_seconds, updated_at = excluded.updated_at",
            params![
                session.session_id,
                started_at.to_rfc3339(),
                session.ended_at.map(|value| value.to_rfc3339()),
                session.project_id,
                session.project_name,
                session.project_path,
                session.model,
                tokens.input_tokens,
                tokens.output_tokens,
                tokens.cached_input_tokens,
                tokens.reasoning_tokens,
                tokens.total_tokens,
                session.requests.max(0),
                session.active_seconds.max(0),
                Utc::now().to_rfc3339(),
            ],
        )?;

        transaction.execute(
            "DELETE FROM session_skills WHERE session_id = ?1",
            params![session.session_id],
        )?;
        for skill in &session.skills {
            let tokens = skill.tokens.normalize();
            transaction.execute(
                "INSERT INTO session_skills(
                    session_id, skill_name, invocations, input_tokens, output_tokens,
                    cached_input_tokens, reasoning_tokens, total_tokens, last_used_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    session.session_id,
                    skill.skill_name,
                    skill.invocations.max(0),
                    tokens.input_tokens,
                    tokens.output_tokens,
                    tokens.cached_input_tokens,
                    tokens.reasoning_tokens,
                    tokens.total_tokens,
                    skill.last_used_at.map(|value| value.to_rfc3339()),
                ],
            )?;
        }

        transaction.execute(
            "INSERT INTO ingest_files(
                 source_hash, byte_size, modified_ms, session_id, processed_at, parser_version)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(source_hash) DO UPDATE SET byte_size = excluded.byte_size,
             modified_ms = excluded.modified_ms, session_id = excluded.session_id,
             processed_at = excluded.processed_at, parser_version = excluded.parser_version",
            params![
                source_hash,
                byte_size,
                modified_ms,
                session.session_id,
                Utc::now().to_rfc3339(),
                SESSION_PARSER_VERSION,
            ],
        )?;

        transaction.commit()?;
        Ok(())
    }

    pub fn rebuild_analytics(&self) -> AppResult<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        rebuild_daily_usage(&transaction)?;
        rebuild_project_usage(&transaction)?;
        rebuild_skill_usage(&transaction)?;
        rebuild_model_usage(&transaction)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn clear_local_data(&self) -> AppResult<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute_batch(
            "DELETE FROM session_skills;
             DELETE FROM session_usage;
             DELETE FROM ingest_files;
             DELETE FROM daily_usage;
             DELETE FROM project_usage;
             DELETE FROM skill_usage;
             DELETE FROM model_usage;
             DELETE FROM usage_snapshots;
             DELETE FROM quota_snapshots;",
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn prune(&self, retention_days: u16) -> AppResult<usize> {
        let cutoff = (Utc::now() - chrono::Duration::days(i64::from(retention_days))).to_rfc3339();
        let deleted = self.connection()?.execute(
            "DELETE FROM session_usage WHERE started_at < ?1",
            params![cutoff],
        )?;
        if deleted > 0 {
            self.rebuild_analytics()?;
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::settings::AppSettings;
    use rusqlite::params;

    #[test]
    fn clear_local_data_removes_statistics_and_keeps_settings() {
        let database = Database::open_in_memory().expect("database");
        database.save_settings(&AppSettings::default()).expect("settings");
        {
            let connection = database.connection.lock().expect("connection");
            connection.execute(
                "INSERT INTO session_usage(session_id, started_at, updated_at) VALUES(?1, ?2, ?2)",
                params!["session-1", "2026-08-22T00:00:00Z"],
            ).expect("session");
            connection.execute(
                "INSERT INTO session_skills(session_id, skill_name) VALUES(?1, ?2)",
                params!["session-1", "browser"],
            ).expect("skill");
            connection.execute("INSERT INTO daily_usage(date) VALUES('2026-08-22')", []).expect("daily");
            connection.execute("INSERT INTO usage_snapshots(captured_at, source) VALUES('2026-08-22T00:00:00Z', 'local')", []).expect("snapshot");
        }

        database.clear_local_data().expect("clear local data");

        let connection = database.connection.lock().expect("connection");
        for table in ["session_usage", "session_skills", "daily_usage", "usage_snapshots", "quota_snapshots"] {
            let count: i64 = connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0))
                .expect("count");
            assert_eq!(count, 0, "{table} should be empty");
        }
        let settings = connection
            .query_row("SELECT value_json FROM settings WHERE key = 'app'", [], |row| row.get::<_, String>(0))
            .expect("settings remain");
        assert!(!settings.is_empty());
    }
}

fn rebuild_daily_usage(transaction: &Transaction<'_>) -> AppResult<()> {
    transaction.execute("DELETE FROM daily_usage", [])?;
    transaction.execute_batch(
        "INSERT INTO daily_usage(
            date, input_tokens, output_tokens, cached_input_tokens, reasoning_tokens,
            total_tokens, requests, sessions, active_seconds, source)
         SELECT date(started_at, 'localtime'), SUM(input_tokens), SUM(output_tokens),
                SUM(cached_input_tokens), SUM(reasoning_tokens), SUM(total_tokens),
                SUM(requests), COUNT(*), SUM(active_seconds), 'local'
         FROM session_usage GROUP BY date(started_at, 'localtime');",
    )?;
    Ok(())
}

fn rebuild_project_usage(transaction: &Transaction<'_>) -> AppResult<()> {
    transaction.execute("DELETE FROM project_usage", [])?;
    transaction.execute_batch(
        "INSERT INTO project_usage(
            project_id, project_name, project_path, date, input_tokens, output_tokens,
            cached_input_tokens, reasoning_tokens, total_tokens, requests, sessions,
            active_seconds, first_used_at, last_used_at, source)
         SELECT project_id, MAX(project_name), MAX(project_path), date(started_at, 'localtime'),
                SUM(input_tokens), SUM(output_tokens), SUM(cached_input_tokens),
                SUM(reasoning_tokens), SUM(total_tokens), SUM(requests), COUNT(*),
                SUM(active_seconds), MIN(started_at), MAX(COALESCE(ended_at, started_at)), 'local'
         FROM session_usage WHERE project_id IS NOT NULL
         GROUP BY project_id, date(started_at, 'localtime');",
    )?;
    Ok(())
}

fn rebuild_skill_usage(transaction: &Transaction<'_>) -> AppResult<()> {
    transaction.execute("DELETE FROM skill_usage", [])?;
    transaction.execute_batch(
        "INSERT INTO skill_usage(
            skill_name, project_id, project_name, date, invocations, input_tokens,
            output_tokens, cached_input_tokens, reasoning_tokens, total_tokens,
            last_used_at, source)
         SELECT sk.skill_name, COALESCE(s.project_id, ''), MAX(s.project_name),
                date(s.started_at, 'localtime'), SUM(sk.invocations), SUM(sk.input_tokens),
                SUM(sk.output_tokens), SUM(sk.cached_input_tokens),
                SUM(sk.reasoning_tokens), SUM(sk.total_tokens),
                MAX(COALESCE(sk.last_used_at, s.ended_at, s.started_at)), 'local'
         FROM session_skills sk JOIN session_usage s ON s.session_id = sk.session_id
         GROUP BY sk.skill_name, COALESCE(s.project_id, ''), date(s.started_at, 'localtime');",
    )?;
    Ok(())
}

fn rebuild_model_usage(transaction: &Transaction<'_>) -> AppResult<()> {
    transaction.execute("DELETE FROM model_usage", [])?;
    transaction.execute_batch(
        "INSERT INTO model_usage(
            model, project_id, project_name, date, input_tokens, output_tokens,
            cached_input_tokens, reasoning_tokens, total_tokens, requests, sessions,
            last_used_at, source)
         SELECT model, COALESCE(project_id, ''), MAX(project_name), date(started_at, 'localtime'),
                SUM(input_tokens), SUM(output_tokens), SUM(cached_input_tokens),
                SUM(reasoning_tokens), SUM(total_tokens), SUM(requests), COUNT(*),
                MAX(COALESCE(ended_at, started_at)), 'local'
         FROM session_usage WHERE model IS NOT NULL AND model != ''
         GROUP BY model, COALESCE(project_id, ''), date(started_at, 'localtime');",
    )?;
    Ok(())
}

fn parse_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|value| {
        DateTime::parse_from_rfc3339(&value)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    })
}

fn cutoff_string(range: UsageRange) -> Option<String> {
    range
        .cutoff(chrono::Local::now().date_naive())
        .map(|date| date.to_string())
}
