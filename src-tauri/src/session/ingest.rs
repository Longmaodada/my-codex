use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::{database::Database, error::AppResult};

use super::parser::SafeSessionParser;

const MAX_SESSION_FILE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_JSONL_LINE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestReport {
    pub files_discovered: u64,
    pub files_processed: u64,
    pub files_unchanged: u64,
    pub files_skipped: u64,
    pub sessions_upserted: u64,
    pub valid_metadata_lines: u64,
    pub invalid_or_oversized_lines: u64,
    pub analytics_rebuilt: bool,
}

pub struct SessionIngestor {
    roots: Vec<PathBuf>,
}

impl SessionIngestor {
    pub fn discover() -> Self {
        let root = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))
            .map(|home| home.join("sessions"));
        Self {
            roots: root.into_iter().collect(),
        }
    }

    #[cfg(test)]
    pub fn from_root(root: PathBuf) -> Self {
        Self { roots: vec![root] }
    }

    pub fn ingest(&self, database: &Database) -> AppResult<IngestReport> {
        let mut report = IngestReport::default();
        for root in &self.roots {
            if !root.is_dir() {
                continue;
            }
            for entry in WalkDir::new(root)
                .follow_links(false)
                .max_depth(6)
                .into_iter()
                .filter_map(Result::ok)
            {
                if !entry.file_type().is_file()
                    || entry.path().extension().and_then(|value| value.to_str()) != Some("jsonl")
                {
                    continue;
                }
                report.files_discovered = report.files_discovered.saturating_add(1);
                match self.ingest_file(entry.path(), database, &mut report) {
                    Ok(()) => {}
                    Err(_) => report.files_skipped = report.files_skipped.saturating_add(1),
                }
            }
        }
        if report.sessions_upserted > 0 {
            database.rebuild_analytics()?;
            report.analytics_rebuilt = true;
        }
        Ok(report)
    }

    fn ingest_file(
        &self,
        path: &Path,
        database: &Database,
        report: &mut IngestReport,
    ) -> AppResult<()> {
        let metadata = path.metadata()?;
        if metadata.len() > MAX_SESSION_FILE_BYTES {
            report.files_skipped = report.files_skipped.saturating_add(1);
            return Ok(());
        }
        let modified_ms = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_millis().min(i64::MAX as u128) as i64)
            .unwrap_or_default();
        let source_hash = hash_path(path);
        let byte_size = metadata.len().min(i64::MAX as u64) as i64;
        if database.is_ingest_file_current(&source_hash, byte_size, modified_ms)? {
            report.files_unchanged = report.files_unchanged.saturating_add(1);
            return Ok(());
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut parser = SafeSessionParser::new(source_hash.clone());
        loop {
            match read_bounded_line(&mut reader)? {
                BoundedLine::Eof => break,
                BoundedLine::Oversized => {
                    report.invalid_or_oversized_lines =
                        report.invalid_or_oversized_lines.saturating_add(1);
                }
                BoundedLine::Line(line) => {
                    if line.iter().all(u8::is_ascii_whitespace) {
                        continue;
                    }
                    if parser.ingest_line(&line) {
                        report.valid_metadata_lines = report.valid_metadata_lines.saturating_add(1);
                    } else {
                        report.invalid_or_oversized_lines =
                            report.invalid_or_oversized_lines.saturating_add(1);
                    }
                }
            }
        }
        let session = parser.finish();
        if session.started_at.is_none() {
            report.files_skipped = report.files_skipped.saturating_add(1);
            return Ok(());
        }
        database.upsert_session(
            &session,
            &source_hash,
            byte_size,
            modified_ms,
        )?;
        report.files_processed = report.files_processed.saturating_add(1);
        report.sessions_upserted = report.sessions_upserted.saturating_add(1);
        Ok(())
    }
}

enum BoundedLine {
    Eof,
    Line(Vec<u8>),
    Oversized,
}

fn read_bounded_line<R: BufRead>(reader: &mut R) -> io::Result<BoundedLine> {
    let mut output = Vec::with_capacity(4096);
    let mut oversized = false;
    let mut read_any = false;
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            if !read_any {
                return Ok(BoundedLine::Eof);
            }
            break;
        }
        read_any = true;
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(buffer.len(), |position| position + 1);
        if !oversized && output.len().saturating_add(take) <= MAX_JSONL_LINE_BYTES {
            output.extend_from_slice(&buffer[..take]);
        } else {
            oversized = true;
            output.clear();
        }
        reader.consume(take);
        if newline.is_some() {
            break;
        }
    }
    if oversized {
        return Ok(BoundedLine::Oversized);
    }
    while matches!(output.last(), Some(b'\n' | b'\r')) {
        output.pop();
    }
    Ok(BoundedLine::Line(output))
}

fn hash_path(path: &Path) -> String {
    let mut hasher = Sha256::new();
    let normalized = path.to_string_lossy();
    if cfg!(windows) {
        hasher.update(normalized.to_lowercase().as_bytes());
    } else {
        hasher.update(normalized.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingests_sanitized_fixture_incrementally() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let fixture_path = temp.path().join("safe.jsonl");
        std::fs::write(
            &fixture_path,
            include_bytes!("../../tests/fixtures/session_safe.jsonl"),
        )
        .expect("write fixture");
        let database = Database::open_in_memory().expect("database");
        let ingestor = SessionIngestor::from_root(temp.path().to_path_buf());
        let first = ingestor.ingest(&database).expect("first ingest");
        assert_eq!(first.sessions_upserted, 1);
        let second = ingestor.ingest(&database).expect("second ingest");
        assert_eq!(second.files_unchanged, 1);
        assert_eq!(database.usage_totals(crate::analytics::UsageRange::All).expect("totals").tokens.total_tokens, 150);
    }
}
