use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::models::*;
use crate::scanner;

const TAIL_BYTES: u64 = 16384; // Read last 16KB of JSONL files

/// Get the Claude home directory (~/.claude)
fn claude_home() -> PathBuf {
    dirs_home().join(".claude")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// Read Claude global settings for default model
pub fn read_settings() -> Option<ClaudeSettings> {
    let path = claude_home().join("settings.json");
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Discover Claude sessions from session files
pub fn discover_sessions() -> Vec<ClaudeSessionFile> {
    let sessions_dir = claude_home().join("sessions");
    let pattern = sessions_dir.join("*.json").to_string_lossy().to_string();

    let mut sessions = Vec::new();
    for entry in glob::glob(&pattern).into_iter().flatten() {
        if let Ok(path) = entry {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<ClaudeSessionFile>(&data) {
                    sessions.push(session);
                }
            }
        }
    }
    sessions
}

/// Find the session JSONL file for a given session
fn find_session_jsonl(session_id: &str) -> Option<PathBuf> {
    let projects_dir = claude_home().join("projects");
    if !projects_dir.exists() {
        return None;
    }

    // Search through project directories for the session JSONL
    let pattern = projects_dir
        .join("*")
        .join(format!("{}.jsonl", session_id))
        .to_string_lossy()
        .to_string();

    glob::glob(&pattern)
        .ok()?
        .filter_map(|e| e.ok())
        .next()
}

/// Read the tail of a JSONL file and extract the last usage data
fn read_jsonl_tail(path: &PathBuf) -> Option<(ClaudeUsage, Option<String>, Option<f64>)> {
    let mut file = fs::File::open(path).ok()?;
    let file_len = file.metadata().ok()?.len();

    // Seek to the last TAIL_BYTES
    let seek_pos = if file_len > TAIL_BYTES {
        file_len - TAIL_BYTES
    } else {
        0
    };
    file.seek(SeekFrom::Start(seek_pos)).ok()?;

    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;

    // Parse lines in reverse to find last assistant message with usage
    let mut last_usage: Option<ClaudeUsage> = None;
    let mut last_model: Option<String> = None;
    let mut total_cost: f64 = 0.0;

    for line in buf.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(msg) = serde_json::from_str::<ClaudeJsonlMessage>(line) {
            // Accumulate cost
            if let Some(cost) = msg.cost_usd {
                total_cost += cost;
            }

            // Check for usage in the message wrapper
            if let Some(ref message) = msg.message {
                if let Some(ref model) = message.model {
                    last_model = Some(model.clone());
                }
                if let Some(ref usage) = message.usage {
                    last_usage = Some(ClaudeUsage {
                        input_tokens: usage.input_tokens,
                        output_tokens: usage.output_tokens,
                        cache_read_input_tokens: usage.cache_read_input_tokens,
                        cache_creation_input_tokens: usage.cache_creation_input_tokens,
                    });
                }
            }

            // Also check top-level usage
            if let Some(ref usage) = msg.usage {
                last_usage = Some(ClaudeUsage {
                    input_tokens: usage.input_tokens,
                    output_tokens: usage.output_tokens,
                    cache_read_input_tokens: usage.cache_read_input_tokens,
                    cache_creation_input_tokens: usage.cache_creation_input_tokens,
                });
            }
        }
    }

    let cost = if total_cost > 0.0 { Some(total_cost) } else { None };
    last_usage.map(|u| (u, last_model, cost))
}

/// Determine context window from model string
fn context_window_for_model(model: &str) -> u64 {
    if model.contains("[1m]") || model.contains("1m") {
        1_000_000
    } else if model.contains("haiku") {
        200_000
    } else {
        200_000
    }
}

/// Check how recently a JSONL file was modified (seconds ago)
fn file_modified_ago(path: &PathBuf) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let now = SystemTime::now();
    now.duration_since(modified).ok().map(|d| d.as_secs())
}

/// Check for IDE integration lock files
pub fn check_ide_lock(pid: u32) -> Option<String> {
    let ide_dir = claude_home().join("ide");
    let lock_path = ide_dir.join(format!("{}.lock", pid));
    if lock_path.exists() {
        fs::read_to_string(&lock_path).ok().map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// Build AgentInfo for a Claude session
pub fn build_agent_info(session: &ClaudeSessionFile, settings: &Option<ClaudeSettings>) -> Option<AgentInfo> {
    let pid = session.pid?;
    let cwd = session.cwd.clone().unwrap_or_else(|| "unknown".to_string());

    // Determine if process is alive
    let alive = scanner::is_pid_alive(pid);

    // Get model from settings
    let default_model = settings
        .as_ref()
        .and_then(|s| s.model.clone())
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    // Calculate runtime — startedAt can be epoch millis (number) or RFC3339 (string)
    let runtime = if let Some(ref started_at) = session.started_at {
        match started_at {
            serde_json::Value::Number(n) => {
                // Epoch milliseconds
                if let Some(ms) = n.as_u64() {
                    let started_secs = ms / 1000;
                    let now = scanner::now_epoch();
                    now.saturating_sub(started_secs)
                } else if let Some(ms) = n.as_f64() {
                    let started_secs = (ms / 1000.0) as u64;
                    let now = scanner::now_epoch();
                    now.saturating_sub(started_secs)
                } else {
                    0
                }
            }
            serde_json::Value::String(s) => {
                // RFC3339 string
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    let now = chrono::Utc::now();
                    (now - dt.with_timezone(&chrono::Utc)).num_seconds().max(0) as u64
                } else {
                    0
                }
            }
            _ => 0,
        }
    } else {
        0
    };

    // Try to read JSONL for token usage (don't bail if session_id missing)
    let session_id = session.session_id.as_deref();
    let jsonl_path = session_id.and_then(find_session_jsonl);

    let mut model = default_model;
    let mut tokens: Option<TokenUsage> = None;
    let mut context_percent: Option<f64> = None;
    let mut cost_usd: Option<f64> = None;
    let mut context_window: u64 = 200_000;

    if let Some(ref jpath) = jsonl_path {
        if let Some((usage, found_model, found_cost)) = read_jsonl_tail(jpath) {
            if let Some(m) = found_model {
                model = m;
            }

            context_window = context_window_for_model(&model);

            let input = usage.input_tokens.unwrap_or(0);
            let output = usage.output_tokens.unwrap_or(0);
            let cache_read = usage.cache_read_input_tokens.unwrap_or(0);
            let cache_create = usage.cache_creation_input_tokens.unwrap_or(0);
            let total = input + cache_read + cache_create;

            context_percent = Some((total as f64 / context_window as f64) * 100.0);

            tokens = Some(TokenUsage {
                input_tokens: input,
                output_tokens: output,
                cache_read_tokens: cache_read,
                cache_creation_tokens: cache_create,
                total_tokens: total,
            });

            cost_usd = found_cost;
        }
    }

    // Determine status
    let status = if !alive {
        AgentStatus::Dead
    } else if let Some(ref jpath) = jsonl_path {
        let secs_ago = file_modified_ago(jpath).unwrap_or(999);
        if secs_ago <= 15 {
            AgentStatus::Running
        } else if scanner::is_tty_foreground(pid) {
            AgentStatus::Waiting
        } else {
            AgentStatus::Idle
        }
    } else {
        // No JSONL found — check if foreground
        if scanner::is_tty_foreground(pid) {
            AgentStatus::Waiting
        } else {
            AgentStatus::Idle
        }
    };

    let ide = check_ide_lock(pid);

    Some(AgentInfo {
        agent_type: AgentType::Claude,
        status,
        pid,
        cwd,
        model,
        runtime_seconds: runtime,
        context_percent,
        context_window,
        tokens,
        cost_usd,
        ide,
        session_id: session_id.map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_context_window_1m_bracket_tag() {
        assert_eq!(context_window_for_model("claude-opus-4-6[1m]"), 1_000_000);
    }

    #[test]
    fn test_context_window_1m_plain() {
        assert_eq!(context_window_for_model("claude-opus-4-1m"), 1_000_000);
    }

    #[test]
    fn test_context_window_haiku() {
        assert_eq!(context_window_for_model("claude-haiku-4-5-20251001"), 200_000);
    }

    #[test]
    fn test_context_window_default_sonnet() {
        assert_eq!(context_window_for_model("claude-sonnet-4-20250514"), 200_000);
    }

    #[test]
    fn test_context_window_unknown_model() {
        assert_eq!(context_window_for_model("some-new-model"), 200_000);
    }

    #[test]
    fn test_read_jsonl_tail_single_message() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"assistant","message":{{"role":"assistant","model":"claude-opus-4-20250514","usage":{{"input_tokens":50000,"output_tokens":2000,"cache_read_input_tokens":10000,"cache_creation_input_tokens":5000}}}},"costUSD":0.25}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let result = read_jsonl_tail(&pb);
        assert!(result.is_some());

        let (usage, model, cost) = result.unwrap();
        assert_eq!(usage.input_tokens, Some(50000));
        assert_eq!(usage.output_tokens, Some(2000));
        assert_eq!(usage.cache_read_input_tokens, Some(10000));
        assert_eq!(usage.cache_creation_input_tokens, Some(5000));
        assert_eq!(model.as_deref(), Some("claude-opus-4-20250514"));
        assert!((cost.unwrap() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_read_jsonl_tail_multiple_messages_takes_last() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        // First message
        writeln!(f, r#"{{"message":{{"model":"claude-sonnet-4-20250514","usage":{{"input_tokens":1000,"output_tokens":100}}}},"costUSD":0.01}}"#).unwrap();
        // Second message (should be the one returned)
        writeln!(f, r#"{{"message":{{"model":"claude-opus-4-20250514","usage":{{"input_tokens":9000,"output_tokens":500}}}},"costUSD":0.10}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let (usage, model, cost) = read_jsonl_tail(&pb).unwrap();
        assert_eq!(usage.input_tokens, Some(9000));
        assert_eq!(model.as_deref(), Some("claude-opus-4-20250514"));
        // Cost accumulates both lines
        assert!((cost.unwrap() - 0.11).abs() < 0.001);
    }

    #[test]
    fn test_read_jsonl_tail_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.jsonl");
        fs::File::create(&path).unwrap();

        let pb = PathBuf::from(&path);
        assert!(read_jsonl_tail(&pb).is_none());
    }

    #[test]
    fn test_read_jsonl_tail_no_usage_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"type":"user","role":"user"}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        assert!(read_jsonl_tail(&pb).is_none());
    }

    #[test]
    fn test_read_jsonl_tail_nonexistent_file() {
        let pb = PathBuf::from("/nonexistent/path/to/test.jsonl");
        assert!(read_jsonl_tail(&pb).is_none());
    }

    #[test]
    fn test_read_jsonl_tail_with_top_level_usage() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"usage":{{"input_tokens":7777,"output_tokens":333}}}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let (usage, model, cost) = read_jsonl_tail(&pb).unwrap();
        assert_eq!(usage.input_tokens, Some(7777));
        assert_eq!(usage.output_tokens, Some(333));
        assert!(model.is_none());
        assert!(cost.is_none());
    }

    #[test]
    fn test_file_modified_ago_recent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("recent.txt");
        fs::write(&path, "hello").unwrap();

        let pb = PathBuf::from(&path);
        let ago = file_modified_ago(&pb).unwrap();
        assert!(ago < 5, "file should be modified < 5 seconds ago, got {}", ago);
    }

    #[test]
    fn test_file_modified_ago_nonexistent() {
        let pb = PathBuf::from("/nonexistent/file.txt");
        assert!(file_modified_ago(&pb).is_none());
    }

    #[test]
    fn test_check_ide_lock_nonexistent() {
        // With a random PID, there should be no IDE lock
        assert!(check_ide_lock(4_000_000_000).is_none());
    }

    #[test]
    fn test_discover_sessions_returns_vec() {
        // Should not panic even if ~/.claude/sessions doesn't exist
        let sessions = discover_sessions();
        // Just check it returns (may be empty or have real sessions)
        let _ = sessions.len();
    }

    #[test]
    fn test_read_settings_returns_option() {
        // Should not panic even if ~/.claude/settings.json doesn't exist
        let _settings = read_settings();
    }

    #[test]
    fn test_read_jsonl_large_file_tail_only() {
        // Create a file larger than TAIL_BYTES to verify seek behavior
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large.jsonl");
        let mut f = fs::File::create(&path).unwrap();

        // Write padding lines (junk that won't parse as valid messages)
        for _ in 0..2000 {
            writeln!(f, r#"{{"type":"padding","data":"{}"}}"#, "x".repeat(100)).unwrap();
        }
        // Write the real message at the end
        writeln!(f, r#"{{"message":{{"model":"claude-opus-4-6","usage":{{"input_tokens":42000,"output_tokens":1000}}}},"costUSD":0.50}}"#).unwrap();
        f.flush().unwrap();

        let pb = PathBuf::from(&path);
        let result = read_jsonl_tail(&pb);
        assert!(result.is_some());
        let (usage, model, cost) = result.unwrap();
        assert_eq!(usage.input_tokens, Some(42000));
        assert_eq!(model.as_deref(), Some("claude-opus-4-6"));
        assert!((cost.unwrap() - 0.50).abs() < 0.001);
    }
}
