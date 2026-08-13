pub const MIGRATIONS: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY NOT NULL,
        value_json TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS quota_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        captured_at TEXT NOT NULL,
        provider TEXT NOT NULL,
        status TEXT NOT NULL,
        source TEXT NOT NULL,
        plan TEXT,
        primary_used_percent REAL,
        primary_remaining_percent REAL,
        primary_window_minutes INTEGER,
        primary_resets_at TEXT,
        secondary_used_percent REAL,
        secondary_remaining_percent REAL,
        secondary_window_minutes INTEGER,
        secondary_resets_at TEXT,
        reset_at TEXT,
        input_tokens INTEGER,
        output_tokens INTEGER,
        cached_input_tokens INTEGER,
        reasoning_tokens INTEGER,
        total_tokens INTEGER
    );
    CREATE INDEX IF NOT EXISTS idx_quota_snapshots_captured_at
        ON quota_snapshots(captured_at DESC);

    CREATE TABLE IF NOT EXISTS usage_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        captured_at TEXT NOT NULL,
        source TEXT NOT NULL,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cached_input_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        total_tokens INTEGER NOT NULL DEFAULT 0,
        requests INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS ingest_files (
        source_hash TEXT PRIMARY KEY NOT NULL,
        byte_size INTEGER NOT NULL,
        modified_ms INTEGER NOT NULL,
        session_id TEXT,
        processed_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS session_usage (
        session_id TEXT PRIMARY KEY NOT NULL,
        started_at TEXT NOT NULL,
        ended_at TEXT,
        project_id TEXT,
        project_name TEXT,
        project_path TEXT,
        model TEXT,
        input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(input_tokens >= 0),
        output_tokens INTEGER NOT NULL DEFAULT 0 CHECK(output_tokens >= 0),
        cached_input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(cached_input_tokens >= 0),
        reasoning_tokens INTEGER NOT NULL DEFAULT 0 CHECK(reasoning_tokens >= 0),
        total_tokens INTEGER NOT NULL DEFAULT 0 CHECK(total_tokens >= 0),
        requests INTEGER NOT NULL DEFAULT 0 CHECK(requests >= 0),
        active_seconds INTEGER NOT NULL DEFAULT 0 CHECK(active_seconds >= 0),
        source TEXT NOT NULL DEFAULT 'local',
        updated_at TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_session_usage_started_at
        ON session_usage(started_at DESC);
    CREATE INDEX IF NOT EXISTS idx_session_usage_project_id
        ON session_usage(project_id);

    CREATE TABLE IF NOT EXISTS session_skills (
        session_id TEXT NOT NULL REFERENCES session_usage(session_id) ON DELETE CASCADE,
        skill_name TEXT NOT NULL,
        invocations INTEGER NOT NULL DEFAULT 0 CHECK(invocations >= 0),
        input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(input_tokens >= 0),
        output_tokens INTEGER NOT NULL DEFAULT 0 CHECK(output_tokens >= 0),
        cached_input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(cached_input_tokens >= 0),
        reasoning_tokens INTEGER NOT NULL DEFAULT 0 CHECK(reasoning_tokens >= 0),
        total_tokens INTEGER NOT NULL DEFAULT 0 CHECK(total_tokens >= 0),
        last_used_at TEXT,
        PRIMARY KEY(session_id, skill_name)
    );

    CREATE TABLE IF NOT EXISTS daily_usage (
        date TEXT PRIMARY KEY NOT NULL,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cached_input_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        total_tokens INTEGER NOT NULL DEFAULT 0,
        requests INTEGER NOT NULL DEFAULT 0,
        sessions INTEGER NOT NULL DEFAULT 0,
        active_seconds INTEGER NOT NULL DEFAULT 0,
        source TEXT NOT NULL DEFAULT 'local'
    );

    CREATE TABLE IF NOT EXISTS project_usage (
        project_id TEXT NOT NULL,
        project_name TEXT NOT NULL,
        project_path TEXT,
        date TEXT NOT NULL,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cached_input_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        total_tokens INTEGER NOT NULL DEFAULT 0,
        requests INTEGER NOT NULL DEFAULT 0,
        sessions INTEGER NOT NULL DEFAULT 0,
        active_seconds INTEGER NOT NULL DEFAULT 0,
        first_used_at TEXT,
        last_used_at TEXT,
        source TEXT NOT NULL DEFAULT 'local',
        PRIMARY KEY(project_id, date)
    );
    CREATE INDEX IF NOT EXISTS idx_project_usage_date ON project_usage(date DESC);

    CREATE TABLE IF NOT EXISTS skill_usage (
        skill_name TEXT NOT NULL,
        project_id TEXT NOT NULL DEFAULT '',
        project_name TEXT,
        date TEXT NOT NULL,
        invocations INTEGER NOT NULL DEFAULT 0,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cached_input_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        total_tokens INTEGER NOT NULL DEFAULT 0,
        last_used_at TEXT,
        source TEXT NOT NULL DEFAULT 'local',
        PRIMARY KEY(skill_name, project_id, date)
    );
    CREATE INDEX IF NOT EXISTS idx_skill_usage_date ON skill_usage(date DESC);

    CREATE TABLE IF NOT EXISTS model_usage (
        model TEXT NOT NULL,
        project_id TEXT NOT NULL DEFAULT '',
        project_name TEXT,
        date TEXT NOT NULL,
        input_tokens INTEGER NOT NULL DEFAULT 0,
        output_tokens INTEGER NOT NULL DEFAULT 0,
        cached_input_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        total_tokens INTEGER NOT NULL DEFAULT 0,
        requests INTEGER NOT NULL DEFAULT 0,
        sessions INTEGER NOT NULL DEFAULT 0,
        last_used_at TEXT,
        source TEXT NOT NULL DEFAULT 'local',
        PRIMARY KEY(model, project_id, date)
    );
    CREATE INDEX IF NOT EXISTS idx_model_usage_date ON model_usage(date DESC);

    PRAGMA user_version = 1;
    "#,
];
