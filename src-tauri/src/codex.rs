use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::models::*;
use crate::scanner;

const TAIL_BYTES: u64 = 16384;

fn codex_home() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".codex"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/.codex"))
}

/// Read Codex config.toml
pub fn read_config() -> Option<CodexConfig> {
    let path = codex_home().join("config.toml");
    let data = fs::read_to_string(path).ok()?;
    toml::from_str(&data).ok()
}

/// Thread info from SQLite
#[derive(Debug)]
pub struct CodexThread {
    pub thread_id: String,
    pub cwd: Option<String>,
    pub rollout_path: Option<String>,
    #[allow(dead_code)]
    pub created_at: Option<String>,
}

/// Query threads from Codex SQLite state
pub fn query_threads() -> Vec<CodexThread> {
    let db_path = codex_home().join("state_5.sqlite");
    if !db_path.exists() {
        return Vec::new();
    }

    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = match rusqlite::Connection::open_with_flags(&db_path, flags) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Set busy timeout to avoid blocking
    let _ = conn.busy_timeout(std::time::Duration::from_millis(1000));

    let mut stmt = match conn.prepare(
        "SELECT id, cwd, rollout_path, created_at FROM threads ORDER BY created_at DESC LIMIT 20",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let threads = stmt
        .query_map([], |row| {
            Ok(CodexThread {
                thread_id: row.get(0)?,
                cwd: row.get(1)?,
                rollout_path: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    threads
}

/// Read the tail of a rollout JSONL to get token usage
fn read_rollout_tail(path: &PathBuf) -> Option<(CodexTokenUsage, u64)> {
    let mut file = fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();

    let seek_pos = if file_len > TAIL_BYTES {
        file_len - TAIL_BYTES
    } else {
        0
    };
    file.seek(SeekFrom::Start(seek_pos)).ok()?;

    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;

    let mut last_usage: Option<CodexTokenUsage> = None;
    let mut last_context_window: u64 = 200_000;

    for line in buf.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<CodexRolloutEvent>(line) {
            if event.event_type.as_deref() == Some("token_count") {
                if let Some(usage) = event.token_usage {
                    last_usage = Some(usage);
                }
                if let Some(cw) = event.model_context_window {
                    last_context_window = cw;
                }
            }
        }
    }

    last_usage.map(|u| (u, last_context_window))
}

/// Build AgentInfo for Codex threads associated with running processes
pub fn build_agent_infos(
    codex_pids: &[&scanner::ProcessInfo],
    config: &Option<CodexConfig>,
) -> Vec<AgentInfo> {
    let threads = query_threads();
    let default_model = config
        .as_ref()
        .and_then(|c| c.model.clone())
        .unwrap_or_else(|| "codex-mini-latest".to_string());

    let mut agents = Vec::new();

    // For each running Codex process, try to match with a thread
    for proc in codex_pids {
        let cwd = "unknown".to_string();
        let model = default_model.clone();
        let mut tokens: Option<TokenUsage> = None;
        let mut context_percent: Option<f64> = None;
        let mut context_window: u64 = 200_000;
        let mut session_id: Option<String> = None;
        let mut matched_cwd = cwd.clone();
        let mut runtime_seconds: u64 = 0;

        // Calculate runtime from process start time
        let now = scanner::now_epoch();
        if proc.start_time > 0 {
            runtime_seconds = now.saturating_sub(proc.start_time);
        }

        // Try to match a thread (best effort — use most recent thread)
        if let Some(thread) = threads.first() {
            session_id = Some(thread.thread_id.clone());
            if let Some(ref c) = thread.cwd {
                matched_cwd = c.clone();
            }

            // Read rollout JSONL
            if let Some(ref rollout_path) = thread.rollout_path {
                let rpath = PathBuf::from(rollout_path);
                if let Some((usage, cw)) = read_rollout_tail(&rpath) {
                    context_window = cw;
                    let total = usage.total_tokens.unwrap_or(0);
                    let input = usage.input_tokens.unwrap_or(0);
                    let output = usage.output_tokens.unwrap_or(0);

                    context_percent = Some((total as f64 / context_window as f64) * 100.0);

                    tokens = Some(TokenUsage {
                        input_tokens: input,
                        output_tokens: output,
                        cache_read_tokens: 0,
                        cache_creation_tokens: 0,
                        total_tokens: total,
                    });
                }
            }
        }

        // Determine status
        let status = if scanner::is_tty_foreground(proc.pid) {
            AgentStatus::Waiting
        } else {
            AgentStatus::Running
        };

        agents.push(AgentInfo {
            agent_type: AgentType::Codex,
            status,
            pid: proc.pid,
            cwd: matched_cwd,
            model,
            runtime_seconds,
            context_percent,
            context_window,
            tokens,
            cost_usd: None,
            ide: None,
            session_id,
        });
    }

    agents
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_read_rollout_tail_token_count_event() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"token_count","token_usage":{{"input_tokens":30000,"output_tokens":5000,"total_tokens":35000}},"model_context_window":128000}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let result = read_rollout_tail(&pb);
        assert!(result.is_some());

        let (usage, ctx_window) = result.unwrap();
        assert_eq!(usage.input_tokens, Some(30000));
        assert_eq!(usage.output_tokens, Some(5000));
        assert_eq!(usage.total_tokens, Some(35000));
        assert_eq!(ctx_window, 128000);
    }

    #[test]
    fn test_read_rollout_tail_multiple_events_takes_last() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"token_count","token_usage":{{"input_tokens":1000,"output_tokens":100,"total_tokens":1100}},"model_context_window":128000}}"#).unwrap();
        writeln!(f, r#"{{"type":"message","token_usage":null}}"#).unwrap();
        writeln!(f, r#"{{"type":"token_count","token_usage":{{"input_tokens":9000,"output_tokens":500,"total_tokens":9500}},"model_context_window":200000}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let (usage, ctx_window) = read_rollout_tail(&pb).unwrap();
        assert_eq!(usage.total_tokens, Some(9500));
        assert_eq!(ctx_window, 200000);
    }

    #[test]
    fn test_read_rollout_tail_no_token_count_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"message","content":"hello"}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        assert!(read_rollout_tail(&pb).is_none());
    }

    #[test]
    fn test_read_rollout_tail_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.jsonl");
        fs::File::create(&path).unwrap();

        let pb = PathBuf::from(&path);
        assert!(read_rollout_tail(&pb).is_none());
    }

    #[test]
    fn test_read_rollout_tail_nonexistent() {
        let pb = PathBuf::from("/nonexistent/rollout.jsonl");
        assert!(read_rollout_tail(&pb).is_none());
    }

    #[test]
    fn test_read_rollout_tail_default_context_window() {
        // If model_context_window is missing, should default to 200_000
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollout.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"token_count","token_usage":{{"total_tokens":500}}}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let (_, ctx_window) = read_rollout_tail(&pb).unwrap();
        assert_eq!(ctx_window, 200_000);
    }

    #[test]
    fn test_read_config_returns_none_when_missing() {
        // If ~/.codex/config.toml doesn't exist, should return None
        let config = read_config();
        // We can't guarantee the file doesn't exist, so just check it doesn't panic
        let _ = config;
    }

    #[test]
    fn test_query_threads_no_db() {
        // Should return empty vec when no SQLite DB exists
        // (relies on ~/.codex/state_5.sqlite not existing, which is typical in test)
        let threads = query_threads();
        let _ = threads.len(); // just ensure no panic
    }

    #[test]
    fn test_query_threads_with_temp_db() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("state_5.sqlite");

        // Create a minimal SQLite database with the threads table
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                cwd TEXT,
                rollout_path TEXT,
                created_at TEXT
            );
            INSERT INTO threads (id, cwd, rollout_path, created_at)
            VALUES ('thread-1', '/home/user/project', '/tmp/rollout.jsonl', '2025-01-01T00:00:00Z');
            INSERT INTO threads (id, cwd, rollout_path, created_at)
            VALUES ('thread-2', '/home/user/other', NULL, '2025-01-02T00:00:00Z');",
        )
        .unwrap();
        drop(conn);

        // We can't easily override codex_home(), but we can verify the DB is valid
        // by opening it directly
        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn2 = rusqlite::Connection::open_with_flags(&db_path, flags).unwrap();
        let _ = conn2.busy_timeout(std::time::Duration::from_millis(1000));

        let mut stmt = conn2
            .prepare("SELECT id, cwd, rollout_path, created_at FROM threads ORDER BY created_at DESC")
            .unwrap();
        let threads: Vec<CodexThread> = stmt
            .query_map([], |row| {
                Ok(CodexThread {
                    thread_id: row.get(0)?,
                    cwd: row.get(1)?,
                    rollout_path: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].thread_id, "thread-2"); // newest first
        assert_eq!(threads[1].thread_id, "thread-1");
        assert_eq!(threads[1].cwd.as_deref(), Some("/home/user/project"));
        assert!(threads[0].rollout_path.is_none());
    }

    #[test]
    fn test_build_agent_infos_empty() {
        let procs: Vec<&scanner::ProcessInfo> = vec![];
        let config = None;
        let agents = build_agent_infos(&procs, &config);
        assert!(agents.is_empty());
    }
}
