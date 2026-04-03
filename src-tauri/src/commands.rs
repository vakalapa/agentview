use crate::claude;
use crate::codex;
use crate::models::*;
use crate::scanner;
use std::collections::HashSet;

#[tauri::command]
pub fn get_agents() -> Vec<AgentInfo> {
    let mut agents: Vec<AgentInfo> = Vec::new();

    // --- Claude Code ---
    let claude_settings = claude::read_settings();
    let sessions = claude::discover_sessions();

    // Track PIDs we've already handled via session files
    let mut seen_pids: HashSet<u32> = HashSet::new();

    for session in &sessions {
        match claude::build_agent_info(session, &claude_settings) {
            Some(info) => {
                seen_pids.insert(info.pid);
                agents.push(info);
            }
            None => {}
        }
    }

    // --- Codex ---
    let codex_config = codex::read_config();

    // Scan running processes
    let processes = scanner::scan_agent_processes();

    let codex_procs: Vec<&scanner::ProcessInfo> = processes
        .iter()
        .filter(|p| p.agent_type == AgentType::Codex)
        .collect();

    if !codex_procs.is_empty() {
        let codex_agents = codex::build_agent_infos(&codex_procs, &codex_config);
        agents.extend(codex_agents);
    }

    // Also pick up any Claude processes not covered by session files
    for proc in &processes {
        if proc.agent_type == AgentType::Claude && !seen_pids.contains(&proc.pid) {
            let now = scanner::now_epoch();
            let runtime = now.saturating_sub(proc.start_time);

            agents.push(AgentInfo {
                agent_type: AgentType::Claude,
                status: if scanner::is_tty_foreground(proc.pid) {
                    AgentStatus::Waiting
                } else {
                    AgentStatus::Running
                },
                pid: proc.pid,
                cwd: "unknown".to_string(),
                model: claude_settings
                    .as_ref()
                    .and_then(|s| s.model.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                runtime_seconds: runtime,
                context_percent: None,
                context_window: 200_000,
                tokens: None,
                cost_usd: None,
                ide: None,
                session_id: None,
            });
        }
    }

    // Sort: Running first, then Waiting, then Idle, then Dead
    agents.sort_by_key(|a| match a.status {
        AgentStatus::Running => 0,
        AgentStatus::Waiting => 1,
        AgentStatus::Idle => 2,
        AgentStatus::Dead => 3,
    });

    agents
}
