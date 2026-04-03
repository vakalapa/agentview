use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Waiting,
    Idle,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub pid: u32,
    pub cwd: String,
    pub model: String,
    pub runtime_seconds: u64,
    pub context_percent: Option<f64>,
    pub context_window: u64,
    pub tokens: Option<TokenUsage>,
    pub cost_usd: Option<f64>,
    pub ide: Option<String>,
    pub session_id: Option<String>,
}

// --- Claude Code deserialization structs ---

#[derive(Debug, Deserialize)]
pub struct ClaudeSessionFile {
    pub pid: Option<u32>,
    pub cwd: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<serde_json::Value>,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeSettings {
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClaudeJsonlMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub role: Option<String>,
    pub message: Option<ClaudeMessage>,
    pub usage: Option<ClaudeUsage>,
    #[serde(rename = "costUSD")]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClaudeMessage {
    pub role: Option<String>,
    pub model: Option<String>,
    pub usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
}

// --- Codex deserialization structs ---

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CodexConfig {
    pub model: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodexRolloutEvent {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub token_usage: Option<CodexTokenUsage>,
    pub model_context_window: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CodexTokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_serialization() {
        let claude = AgentType::Claude;
        let json = serde_json::to_string(&claude).unwrap();
        assert_eq!(json, r#""claude""#);

        let codex = AgentType::Codex;
        let json = serde_json::to_string(&codex).unwrap();
        assert_eq!(json, r#""codex""#);
    }

    #[test]
    fn test_agent_type_deserialization() {
        let claude: AgentType = serde_json::from_str(r#""claude""#).unwrap();
        assert_eq!(claude, AgentType::Claude);

        let codex: AgentType = serde_json::from_str(r#""codex""#).unwrap();
        assert_eq!(codex, AgentType::Codex);
    }

    #[test]
    fn test_agent_status_roundtrip() {
        for status in [AgentStatus::Running, AgentStatus::Waiting, AgentStatus::Idle, AgentStatus::Dead] {
            let json = serde_json::to_string(&status).unwrap();
            let back: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 200,
            cache_creation_tokens: 100,
            total_tokens: 1300,
        };
        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"input_tokens\":1000"));
        assert!(json.contains("\"total_tokens\":1300"));
    }

    #[test]
    fn test_agent_info_full_roundtrip() {
        let info = AgentInfo {
            agent_type: AgentType::Claude,
            status: AgentStatus::Running,
            pid: 12345,
            cwd: "/home/user/project".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            runtime_seconds: 3600,
            context_percent: Some(42.5),
            context_window: 200_000,
            tokens: Some(TokenUsage {
                input_tokens: 85000,
                output_tokens: 5000,
                cache_read_tokens: 10000,
                cache_creation_tokens: 2000,
                total_tokens: 97000,
            }),
            cost_usd: Some(0.15),
            ide: Some("vscode".to_string()),
            session_id: Some("abc-123".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        let back: AgentInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(back.agent_type, AgentType::Claude);
        assert_eq!(back.status, AgentStatus::Running);
        assert_eq!(back.pid, 12345);
        assert_eq!(back.context_percent, Some(42.5));
        assert_eq!(back.tokens.as_ref().unwrap().input_tokens, 85000);
    }

    #[test]
    fn test_agent_info_optional_fields_null() {
        let info = AgentInfo {
            agent_type: AgentType::Codex,
            status: AgentStatus::Idle,
            pid: 99,
            cwd: "/tmp".to_string(),
            model: "codex-mini".to_string(),
            runtime_seconds: 0,
            context_percent: None,
            context_window: 200_000,
            tokens: None,
            cost_usd: None,
            ide: None,
            session_id: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"context_percent\":null"));
        assert!(json.contains("\"tokens\":null"));
    }

    #[test]
    fn test_claude_session_file_deserialization_string() {
        let json = r#"{
            "pid": 42,
            "cwd": "/home/user",
            "startedAt": "2025-01-01T00:00:00Z",
            "sessionId": "sess-abc"
        }"#;
        let session: ClaudeSessionFile = serde_json::from_str(json).unwrap();
        assert_eq!(session.pid, Some(42));
        assert_eq!(session.cwd.as_deref(), Some("/home/user"));
        assert_eq!(session.session_id.as_deref(), Some("sess-abc"));
        // startedAt as string
        assert!(session.started_at.unwrap().is_string());
    }

    #[test]
    fn test_claude_session_file_deserialization_epoch_millis() {
        let json = r#"{
            "pid": 20367,
            "cwd": "/Users/user/project",
            "startedAt": 1775255093381,
            "sessionId": "f7725427-6ecb-4f5d-b1fc-cac203fda92f"
        }"#;
        let session: ClaudeSessionFile = serde_json::from_str(json).unwrap();
        assert_eq!(session.pid, Some(20367));
        // startedAt as number
        let val = session.started_at.unwrap();
        assert!(val.is_number());
        assert_eq!(val.as_u64(), Some(1775255093381));
    }

    #[test]
    fn test_claude_session_file_missing_fields() {
        let json = r#"{}"#;
        let session: ClaudeSessionFile = serde_json::from_str(json).unwrap();
        assert_eq!(session.pid, None);
        assert_eq!(session.cwd, None);
        assert_eq!(session.started_at, None);
    }

    #[test]
    fn test_claude_jsonl_message_with_usage() {
        let json = r#"{
            "type": "assistant",
            "role": "assistant",
            "message": {
                "role": "assistant",
                "model": "claude-opus-4-20250514",
                "usage": {
                    "input_tokens": 50000,
                    "output_tokens": 2000,
                    "cache_read_input_tokens": 10000,
                    "cache_creation_input_tokens": 5000
                }
            },
            "costUSD": 0.25
        }"#;
        let msg: ClaudeJsonlMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.cost_usd, Some(0.25));
        let message = msg.message.unwrap();
        assert_eq!(message.model.as_deref(), Some("claude-opus-4-20250514"));
        let usage = message.usage.unwrap();
        assert_eq!(usage.input_tokens, Some(50000));
        assert_eq!(usage.cache_read_input_tokens, Some(10000));
    }

    #[test]
    fn test_claude_usage_partial() {
        let json = r#"{"input_tokens": 100}"#;
        let usage: ClaudeUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, Some(100));
        assert_eq!(usage.output_tokens, None);
        assert_eq!(usage.cache_read_input_tokens, None);
    }

    #[test]
    fn test_codex_config_deserialization() {
        let toml_str = r#"
            model = "gpt-4o"
            provider = "openai"
        "#;
        let config: CodexConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_codex_rollout_event_token_count() {
        let json = r#"{
            "type": "token_count",
            "token_usage": {
                "input_tokens": 30000,
                "output_tokens": 5000,
                "total_tokens": 35000
            },
            "model_context_window": 128000
        }"#;
        let event: CodexRolloutEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type.as_deref(), Some("token_count"));
        assert_eq!(event.model_context_window, Some(128000));
        let usage = event.token_usage.unwrap();
        assert_eq!(usage.total_tokens, Some(35000));
    }

    #[test]
    fn test_codex_rollout_event_non_token_type() {
        let json = r#"{"type": "message", "token_usage": null}"#;
        let event: CodexRolloutEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type.as_deref(), Some("message"));
        assert!(event.token_usage.is_none());
        assert!(event.model_context_window.is_none());
    }
}
