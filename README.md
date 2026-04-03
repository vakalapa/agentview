# AgentView

A floating, semi-transparent macOS HUD overlay that monitors all running AI coding agents (Claude Code, Codex CLI) across terminals and VSCode in real time.

Built with **Tauri v2 + Svelte 5 + Rust** for a lightweight native experience with a cyberpunk aesthetic.

![macOS](https://img.shields.io/badge/platform-macOS-blue)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-orange)
![Svelte 5](https://img.shields.io/badge/Svelte-5-red)

## Features

- **Auto-detects** Claude Code and Codex CLI sessions across all terminals
- **Real-time status** — Running, Waiting for Input, Idle, Dead
- **Context window usage** — animated progress bar with color gradient (green → amber → red)
- **Token counts** — input, output, cache read/creation
- **Cost tracking** — accumulated USD cost per Claude session
- **Model display** — shows which model each agent is using
- **Runtime** — how long each session has been active
- **IDE integration** — detects VSCode-connected sessions
- **Always-on-top** transparent overlay with backdrop blur
- **Collapsible cards** — click to expand/collapse agent details
- **5-second polling** — lightweight background refresh

## Architecture

```
Frontend (Svelte/TS)          Backend (Rust/Tauri)
┌──────────────┐    invoke    ┌──────────────────────┐
│ 5s interval  │───────────►  │ get_agents() command  │
│ agents store │◄───────────  │                       │
│ HUD cards    │   AgentInfo[]│ ├─ scanner.rs (sysinfo)│
│ context bars │              │ ├─ claude.rs (files)   │
│ status badges│              │ └─ codex.rs (sqlite)   │
└──────────────┘              └──────────────────────┘
```

**Data flow:** Frontend polls every 5s → Rust scans processes via `sysinfo` → cross-references with agent state files → returns `AgentInfo[]` → Svelte reactively updates HUD.

## Agent Detection

### Claude Code

| Data | Source |
|------|--------|
| Active sessions (pid, cwd, startedAt) | `~/.claude/sessions/{PID}.json` |
| Model name | `~/.claude/settings.json` |
| Live token usage / context % | Session JSONL (last 16KB tail read) |
| Cost (USD) | Accumulated from JSONL `costUSD` fields |
| IDE integration | `~/.claude/ide/{PID}.lock` |

**Context %** = `(input_tokens + cache_read + cache_creation) / context_window * 100`

### Codex CLI

| Data | Source |
|------|--------|
| Model, provider | `~/.codex/config.toml` |
| Thread data (cwd, rollout path) | `~/.codex/state_5.sqlite` (read-only) |
| Live tokens + context window | Rollout JSONL `token_count` events |

### Status Detection

| Status | Condition |
|--------|-----------|
| **Running** | JSONL modified within last 15 seconds |
| **Waiting** | Process is TTY foreground AND JSONL stale >15s |
| **Idle** | Process alive but not foreground |
| **Dead** | PID no longer exists |

## Prerequisites

- **macOS** (uses macOS private APIs for window transparency)
- **Rust** (1.77.2+)
- **Node.js** (18+)

## Setup

Install Rust if not already installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

Install dependencies:

```bash
cd ~/workspace/agentview
npm install
```

## Usage

### Development

```bash
npm run tauri dev
```

The HUD window appears as a transparent overlay, always on top, draggable by the title bar.

### Production Build

```bash
npm run tauri build
```

The built `.app` bundle will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
agentview/
├── src/                          # Svelte frontend
│   ├── App.svelte                # Root component
│   ├── main.ts                   # Entry point
│   ├── styles.css                # Global HUD theme
│   └── lib/
│       ├── types/agent.ts        # TypeScript interfaces
│       ├── stores/agents.ts      # Polling store (5s interval)
│       ├── utils/format.ts       # Duration/token formatters
│       └── components/
│           ├── TitleBar.svelte   # Draggable titlebar
│           ├── AgentCard.svelte  # Agent info card
│           ├── StatusBadge.svelte# Status indicator
│           └── ContextBar.svelte # Context usage bar
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Window config (transparent, always-on-top)
│   └── src/
│       ├── lib.rs                # Tauri builder + module declarations
│       ├── main.rs               # Entry point
│       ├── models.rs             # Data structures
│       ├── scanner.rs            # Process detection (sysinfo)
│       ├── claude.rs             # Claude Code state parsing
│       ├── codex.rs              # Codex CLI state/SQLite parsing
│       └── commands.rs           # Tauri command orchestration
├── index.html
├── vite.config.ts
└── package.json
```

## Technical Notes

- **JSONL tail reading** — Only reads the last 16KB of session files to avoid performance issues with multi-MB logs
- **SQLite read-only** — Opens Codex database with `SQLITE_OPEN_READ_ONLY` + `busy_timeout(1000)` to never interfere with running agents
- **PID validation** — Always verifies PIDs are alive before trusting session files; stale sessions show as Dead
- **Graceful degradation** — If any data source fails, the agent still appears with available info; context shows "N/A" instead of crashing

## License

MIT
