# AgentView

A floating, semi-transparent macOS HUD overlay that monitors all running AI coding agents (Claude Code, Codex CLI) across terminals and VSCode in real time.

Built with **Tauri v2 + Svelte 5 + Rust** for a lightweight native experience with a cyberpunk aesthetic.

![macOS](https://img.shields.io/badge/platform-macOS-blue)
![Tauri v2](https://img.shields.io/badge/Tauri-v2-orange)
![Svelte 5](https://img.shields.io/badge/Svelte-5-red)

## Features

- **Auto-detects** Claude Code and Codex CLI sessions across all terminals
- **Real-time status** — Running, Permission, Question, Waiting, Idle, Dead
- **Context window usage** — animated progress bar with color gradient (green → amber → red)
- **Token counts** — input, output, cache read/creation (Claude); total tokens (Codex)
- **Cost tracking** — accumulated USD cost per Claude session
- **Model display** — shows which model each agent is using (e.g. `claude-opus-4-6[1m]`, `gpt-5.4`)
- **Runtime** — how long each session has been active
- **IDE integration** — detects VSCode-connected Claude sessions
- **Desktop app filtering** — ignores Claude.app (desktop), only tracks Claude Code CLI
- **Always-on-top** transparent overlay with backdrop blur
- **Collapsible cards** — click to expand/collapse agent details
- **5-second polling** — lightweight background refresh

## Architecture

```
Frontend (Svelte 5 / TS)       Backend (Rust / Tauri v2)
┌──────────────────┐  invoke   ┌─────────────────────────┐
│ 5s setInterval   │─────────► │ get_agents() command     │
│ agents store     │◄───────── │                          │
│ AgentCard        │ AgentInfo │ ├─ scanner.rs             │
│ StatusBadge      │    []     │ │  └─ sysinfo + lsof      │
│ ContextBar       │           │ ├─ claude.rs              │
│ TitleBar         │           │ │  └─ session files + JSONL│
└──────────────────┘           │ └─ codex.rs               │
                               │    └─ SQLite + config.toml │
                               └─────────────────────────┘
```

**Data flow:**

1. Frontend polls every 5 seconds via Tauri `invoke("get_agents")`
2. Rust scans running processes via `sysinfo` crate — filters by process name and exe path
3. Claude: cross-references process PIDs with `~/.claude/sessions/*.json` files, reads JSONL tails for live token/cost data
4. Codex: reads `~/.codex/state_5.sqlite` (read-only) for thread metadata, tokens, and model
5. Returns `AgentInfo[]` — Svelte reactively updates the HUD cards

## Agent Detection

### Claude Code

| Data | Source | Method |
|------|--------|--------|
| Running sessions | `~/.claude/sessions/{PID}.json` | JSON parse for pid, cwd, startedAt, sessionId |
| Default model | `~/.claude/settings.json` | `model` field |
| Live model + tokens | `~/.claude/projects/{dir}/{sessionId}.jsonl` | Tail-read last 16KB, parse `message.usage` |
| Cost (USD) | Same JSONL | Accumulated from `costUSD` fields across all messages |
| Context % | Computed | `(input + cache_read + cache_creation) / context_window * 100` |
| Context window | Model string | 1M for models containing `1m`; 200K otherwise |
| Wait state | Same JSONL | Last entry's `stop_reason` — see Status Detection |
| IDE integration | `~/.claude/ide/{PID}.lock` | File existence + contents |

**Process filtering:** Scanner matches processes with exact name `claude` but **excludes** any whose exe path contains `Claude.app` (the desktop Electron app). This prevents the Claude desktop app from showing up as a coding agent.

**Sub-agents:** Claude Code's `Agent` tool spawns sub-agents within the same process. They share the parent's PID, JSONL, and session — AgentView does not show them as separate cards. Their token usage is included in the parent session's totals, so context % and cost still reflect real consumption.

### Codex CLI

| Data | Source | Method |
|------|--------|--------|
| Default model, provider | `~/.codex/config.toml` | TOML parse |
| Thread metadata | `~/.codex/state_5.sqlite` | Read-only SQLite query on `threads` table |
| cwd | SQLite `threads.cwd` | Matched to process by comparing process cwd (via `lsof`) against thread cwd |
| Tokens used | SQLite `threads.tokens_used` | Total token count per thread |
| Model | SQLite `threads.model` | Per-thread model (e.g. `gpt-5.4`), falls back to config.toml |
| Context window | Model string | 1M for `gpt-5*`; 200K otherwise |
| Context % | Computed | `tokens_used / context_window * 100` |

**Process cwd detection:** `sysinfo::Process::cwd()` often returns `None` on macOS due to entitlement restrictions. AgentView falls back to `lsof -a -d cwd -p {PID}` which reliably returns the working directory. This cwd is then matched against Codex's SQLite `threads.cwd` to find the correct thread for the running process.

**SQLite access:** Codex uses WAL journal mode. AgentView opens with `SQLITE_OPEN_READ_ONLY` + `busy_timeout(1000ms)` to never block or interfere with the running agent. The `threads` table columns used: `id`, `cwd`, `rollout_path`, `tokens_used`, `model`, `updated_at`.

### Status Detection

#### Claude Code

| Status | Badge Color | Condition |
|--------|-------------|-----------|
| **Running** | Green (pulsing) | JSONL modified within last 15 seconds AND last entry is NOT a pending tool_use |
| **Permission** | Orange (pulsing) | Last JSONL entry is `stop_reason: "tool_use"` with a tool name other than `AskUserQuestion` |
| **Question** | Blue (pulsing) | Last JSONL entry is `stop_reason: "tool_use"` with tool name `AskUserQuestion` |
| **Waiting** | Amber (pulsing) | Process is TTY foreground AND JSONL stale >15s (generic input wait) |
| **Idle** | Gray | Process alive but not TTY foreground, JSONL stale |
| **Dead** | Red | PID no longer exists |

**How permission/question detection works:** When Claude emits a tool call (e.g. `Bash`, `Edit`, `Write`), it writes an assistant message with `stop_reason: "tool_use"` to the JSONL and then waits for the user to approve or deny. AgentView reads the last 4KB of the JSONL, checks if the final entry is an assistant message with this stop reason, and extracts the tool name to determine if it's a permission prompt or a user question.

#### Codex CLI

| Status | Condition |
|--------|-----------|
| **Running** | Process alive and not TTY foreground |
| **Waiting** | Process is TTY foreground |

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

### Build & Install

```bash
make build        # Build AgentView.app (~8MB self-contained binary)
make install      # Build and copy to /Applications
make clean        # Remove build artifacts
```

The `.app` bundle is output to `src-tauri/target/release/bundle/macos/AgentView.app`. You can move it anywhere — it's fully self-contained with no external dependencies.

### Development

```bash
make dev
```

The HUD window appears as a transparent overlay, always on top, draggable by the title bar. Tauri watches `src-tauri/` for Rust changes and hot-reloads the frontend via Vite.

### Tests

```bash
cd src-tauri && cargo test
```

46 unit tests covering: model detection, JSONL parsing, SQLite queries, token calculations, process scanning, session file deserialization.

## Project Structure

```
agentview/
├── Makefile                      # build / install / dev / clean
├── src/                          # Svelte 5 frontend
│   ├── App.svelte                # Root component with drag zones
│   ├── main.ts                   # Entry point
│   ├── styles.css                # Global HUD theme (cyberpunk)
│   └── lib/
│       ├── types/agent.ts        # TypeScript interfaces (AgentInfo, TokenUsage)
│       ├── stores/agents.ts      # Polling store (5s interval via Tauri invoke)
│       ├── utils/format.ts       # Duration/token formatters
│       └── components/
│           ├── TitleBar.svelte   # Draggable titlebar
│           ├── AgentCard.svelte  # Agent info card (collapsible)
│           ├── StatusBadge.svelte# Status indicator with wait reason
│           └── ContextBar.svelte # Context usage bar (animated gradient)
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml                # Dependencies: sysinfo, rusqlite, serde, glob, chrono, toml
│   ├── tauri.conf.json           # Window config (transparent, always-on-top, no decorations)
│   └── src/
│       ├── lib.rs                # Tauri builder + module declarations
│       ├── main.rs               # Entry point (#![cfg_attr(not(debug), windows_subsystem)])
│       ├── models.rs             # Shared data structures (AgentInfo, TokenUsage, enums)
│       ├── scanner.rs            # Process detection (sysinfo), lsof cwd fallback, PID checks
│       ├── claude.rs             # Claude session/JSONL parsing, wait state detection
│       ├── codex.rs              # Codex SQLite queries, config parsing, context calculation
│       └── commands.rs           # Tauri command: get_agents() orchestration
├── index.html
├── vite.config.ts
└── package.json
```

## Technical Notes

- **JSONL tail reading** — Only reads the last 16KB of Claude session files and 4KB for wait-state detection, avoiding performance issues with multi-MB logs
- **SQLite read-only** — Opens Codex database with `SQLITE_OPEN_READ_ONLY` + `busy_timeout(1000)` to never interfere with running agents
- **lsof fallback** — `sysinfo::Process::cwd()` returns `None` on macOS due to entitlement restrictions; `lsof -a -d cwd -p PID` is used as a reliable fallback for process working directory detection
- **Desktop app filtering** — Claude.app (Electron) is excluded by checking the process exe path for `Claude.app`; only the Claude Code CLI binary is tracked
- **PID validation** — Always verifies PIDs are alive via sysinfo before trusting session files; stale sessions show as Dead
- **Thread matching** — Codex threads are matched to running processes by comparing process cwd against `threads.cwd` in SQLite, falling back to the most recently updated thread
- **Graceful degradation** — If any data source fails (JSONL missing, SQLite locked, session file corrupt), the agent still appears with available info; context shows "N/A" instead of crashing

## Known Limitations

- **Claude sub-agents** are not shown as separate cards — they run inside the parent process and share its session/JSONL. Their token usage is reflected in the parent's totals.
- **Codex token breakdown** is not available — Codex only exposes `tokens_used` (total) in its SQLite database, not input/output split.
- **macOS only** — Window transparency and always-on-top use macOS private APIs (`macOSPrivateApi: true` in tauri.conf.json).
- **Polling, not streaming** — The 5-second interval means status changes can take up to 5 seconds to appear. This is a deliberate tradeoff for simplicity and low resource usage.

## License

MIT
