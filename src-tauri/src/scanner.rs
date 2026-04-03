use std::process::Command;
use std::time::SystemTime;
use sysinfo::System;

use crate::models::AgentType;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub agent_type: AgentType,
    pub start_time: u64,
}

/// Scan for running Claude and Codex CLI processes
pub fn scan_agent_processes() -> Vec<ProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut agents = Vec::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        let name_lower = name.to_lowercase();
        let cmd: Vec<String> = process.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect();
        let cmd_joined = cmd.join(" ").to_lowercase();

        let agent_type = if name_lower == "claude" {
            // Exact match on process name "claude" = Claude Code CLI
            // Excludes: Claude.app, Claude Helper, osq_claude_mcp, etc.
            Some(AgentType::Claude)
        } else if name_lower == "codex" || (cmd_joined.contains("/codex") && !cmd_joined.contains("codex-")) {
            Some(AgentType::Codex)
        } else {
            None
        };

        if let Some(agent_type) = agent_type {
            agents.push(ProcessInfo {
                pid: pid.as_u32(),
                agent_type,
                start_time: process.start_time(),
            });
        }
    }

    agents
}

/// Check if a process is in the foreground of its TTY (waiting for input).
/// Returns: true = foreground (potentially waiting), false = background or no TTY
pub fn is_tty_foreground(pid: u32) -> bool {
    let output = Command::new("ps")
        .args(["-o", "pgid=,tpgid=,tty=", "-p", &pid.to_string()])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                // PGID == TPGID means the process group is in the foreground
                parts[0] == parts[1]
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Check if a PID is still alive
pub fn is_pid_alive(pid: u32) -> bool {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]), true);
    sys.process(sysinfo::Pid::from_u32(pid)).is_some()
}

/// Get current epoch seconds
pub fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_epoch_is_recent() {
        let epoch = now_epoch();
        // Should be after 2024-01-01 (1704067200) and before 2030-01-01 (1893456000)
        assert!(epoch > 1_704_067_200, "epoch {} too small", epoch);
        assert!(epoch < 1_893_456_000, "epoch {} too large", epoch);
    }

    #[test]
    fn test_is_pid_alive_with_init() {
        // PID 1 (launchd on macOS) should always be alive
        assert!(is_pid_alive(1));
    }

    #[test]
    fn test_is_pid_alive_with_bogus_pid() {
        // An absurdly high PID should not exist
        assert!(!is_pid_alive(4_000_000_000));
    }

    #[test]
    fn test_is_tty_foreground_bogus_pid() {
        // Non-existent PID should return false
        assert!(!is_tty_foreground(4_000_000_000));
    }

    #[test]
    fn test_scan_agent_processes_returns_vec() {
        // Should not panic; returns some vec (may be empty if no agents running)
        let procs = scan_agent_processes();
        for p in &procs {
            assert!(p.pid > 0);
            assert!(p.agent_type == AgentType::Claude || p.agent_type == AgentType::Codex);
        }
    }
}

