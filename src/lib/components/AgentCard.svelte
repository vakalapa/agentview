<script lang="ts">
  import type { AgentInfo } from "../types/agent";
  import StatusBadge from "./StatusBadge.svelte";
  import ContextBar from "./ContextBar.svelte";
  import { formatDuration, formatTokenCount, shortenPath } from "../utils/format";

  interface Props {
    agent: AgentInfo;
  }

  let { agent }: Props = $props();
  let collapsed = $state(false);

  let borderColor = $derived(
    agent.agent_type === "claude" ? "#00ffaa" : "#a855f7"
  );

  let agentIcon = $derived(
    agent.agent_type === "claude" ? "C" : "X"
  );

  let agentLabel = $derived(
    agent.agent_type === "claude" ? "Claude Code" : "Codex CLI"
  );
</script>

<div
  class="card"
  class:dead={agent.status === "dead"}
  style="border-left-color: {borderColor};"
>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="card-header" role="button" tabindex="0" onclick={() => collapsed = !collapsed} onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') collapsed = !collapsed; }}>
    <div class="header-left">
      <span class="agent-icon" style="background: {borderColor}20; color: {borderColor};">
        {agentIcon}
      </span>
      <div class="header-info">
        <div class="header-top">
          <span class="agent-label">{agentLabel}</span>
          <StatusBadge status={agent.status} />
        </div>
        <div class="cwd" title={agent.cwd}>{shortenPath(agent.cwd)}</div>
      </div>
    </div>
    <div class="header-right">
      <span class="runtime">{formatDuration(agent.runtime_seconds)}</span>
      <span class="collapse-icon">{collapsed ? "+" : "-"}</span>
    </div>
  </div>

  {#if !collapsed}
    <div class="card-body">
      <div class="info-row">
        <span class="info-label">MODEL</span>
        <span class="info-value model">{agent.model}</span>
      </div>

      <div class="info-row">
        <span class="info-label">CONTEXT</span>
      </div>
      <ContextBar
        percent={agent.context_percent}
        totalTokens={agent.tokens?.total_tokens ?? 0}
        contextWindow={agent.context_window}
      />

      {#if agent.tokens}
        <div class="token-grid">
          <div class="token-item">
            <span class="token-label">IN</span>
            <span class="token-value">{formatTokenCount(agent.tokens.input_tokens)}</span>
          </div>
          <div class="token-item">
            <span class="token-label">OUT</span>
            <span class="token-value">{formatTokenCount(agent.tokens.output_tokens)}</span>
          </div>
          {#if agent.tokens.cache_read_tokens > 0}
            <div class="token-item">
              <span class="token-label">CACHE</span>
              <span class="token-value">{formatTokenCount(agent.tokens.cache_read_tokens)}</span>
            </div>
          {/if}
        </div>
      {/if}

      <div class="card-footer">
        <span class="pid">PID {agent.pid}</span>
        {#if agent.cost_usd}
          <span class="cost">${agent.cost_usd.toFixed(4)}</span>
        {/if}
        {#if agent.ide}
          <span class="ide">{agent.ide}</span>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .card {
    background: rgba(18, 22, 36, 0.90);
    border: 1px solid rgba(112, 136, 160, 0.1);
    border-left: 3px solid;
    border-radius: 8px;
    margin-bottom: 8px;
    overflow: hidden;
    transition: border-color 0.3s ease;
  }

  .card:hover {
    border-color: rgba(112, 136, 160, 0.25);
  }

  .card.dead {
    opacity: 0.5;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    cursor: pointer;
    user-select: none;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .agent-icon {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: "JetBrains Mono", monospace;
    font-weight: 700;
    font-size: 14px;
    flex-shrink: 0;
  }

  .header-info {
    min-width: 0;
  }

  .header-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .agent-label {
    font-size: 11px;
    font-weight: 600;
    color: #e0e8f0;
  }

  .cwd {
    font-size: 10px;
    color: #7088a0;
    font-family: "JetBrains Mono", monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
    margin-top: 2px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .runtime {
    font-size: 11px;
    color: #7088a0;
    font-family: "JetBrains Mono", monospace;
  }

  .collapse-icon {
    color: #7088a0;
    font-size: 14px;
    width: 16px;
    text-align: center;
  }

  .card-body {
    padding: 0 12px 10px 12px;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .info-label {
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 1.5px;
    color: #7088a0;
  }

  .info-value {
    font-size: 10px;
    color: #e0e8f0;
    font-family: "JetBrains Mono", monospace;
  }

  .info-value.model {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 220px;
  }

  .token-grid {
    display: flex;
    gap: 12px;
    margin-top: 8px;
  }

  .token-item {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .token-label {
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 1px;
    color: #7088a0;
  }

  .token-value {
    font-size: 12px;
    font-weight: 600;
    color: #e0e8f0;
    font-family: "JetBrains Mono", monospace;
  }

  .card-footer {
    display: flex;
    gap: 12px;
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid rgba(112, 136, 160, 0.08);
  }

  .pid, .cost, .ide {
    font-size: 9px;
    color: #7088a0;
    font-family: "JetBrains Mono", monospace;
  }

  .cost {
    color: #ffbe3c;
  }

  .ide {
    color: #a855f7;
  }
</style>
