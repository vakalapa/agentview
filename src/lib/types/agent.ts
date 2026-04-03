export type AgentType = "claude" | "codex";
export type AgentStatus = "running" | "waiting" | "idle" | "dead";

export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  total_tokens: number;
}

export interface AgentInfo {
  agent_type: AgentType;
  status: AgentStatus;
  pid: number;
  cwd: string;
  model: string;
  runtime_seconds: number;
  context_percent: number | null;
  context_window: number;
  tokens: TokenUsage | null;
  cost_usd: number | null;
  ide: string | null;
  session_id: string | null;
}
