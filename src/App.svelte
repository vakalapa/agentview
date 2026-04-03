<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import AgentCard from "./lib/components/AgentCard.svelte";
  import { agents, lastUpdated, startPolling, stopPolling } from "./lib/stores/agents";
  import { formatTime } from "./lib/utils/format";

  onMount(() => {
    startPolling(5000);
  });

  onDestroy(() => {
    stopPolling();
  });

  type ResizeDirection = "North" | "South" | "East" | "West" | "NorthEast" | "NorthWest" | "SouthEast" | "SouthWest";

  function startResize(direction: ResizeDirection) {
    return (e: MouseEvent) => {
      e.preventDefault();
      getCurrentWindow().startResizeDragging(direction);
    };
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- Resize edge handles -->
<div class="resize-n" role="separator" onmousedown={startResize("North")}></div>
<div class="resize-s" role="separator" onmousedown={startResize("South")}></div>
<div class="resize-e" role="separator" onmousedown={startResize("East")}></div>
<div class="resize-w" role="separator" onmousedown={startResize("West")}></div>
<div class="resize-ne" role="separator" onmousedown={startResize("NorthEast")}></div>
<div class="resize-nw" role="separator" onmousedown={startResize("NorthWest")}></div>
<div class="resize-se" role="separator" onmousedown={startResize("SouthEast")}></div>
<div class="resize-sw" role="separator" onmousedown={startResize("SouthWest")}></div>

<main class="hud-container">
  <TitleBar />

  <div class="content">
    {#if $agents.length === 0}
      <div class="empty-state">
        <div class="empty-icon">&#x25C8;</div>
        <div class="empty-title">No agents detected</div>
        <div class="empty-sub">
          Start a Claude Code or Codex CLI session<br />to see it appear here
        </div>
      </div>
    {:else}
      {#each $agents as agent (agent.pid)}
        <AgentCard {agent} />
      {/each}
    {/if}
  </div>

  <div class="footer">
    <span>Updated {formatTime($lastUpdated)}</span>
    <span class="agent-count">{$agents.length} agent{$agents.length !== 1 ? "s" : ""}</span>
  </div>
</main>

<style>
  .hud-container {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: rgba(10, 12, 20, 0.85);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 255, 170, 0.08);
    border-radius: 12px;
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    scrollbar-width: thin;
    scrollbar-color: rgba(112, 136, 160, 0.2) transparent;
  }

  .content::-webkit-scrollbar {
    width: 4px;
  }

  .content::-webkit-scrollbar-track {
    background: transparent;
  }

  .content::-webkit-scrollbar-thumb {
    background: rgba(112, 136, 160, 0.2);
    border-radius: 2px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    padding: 40px 20px;
  }

  .empty-icon {
    font-size: 40px;
    color: #00ffaa;
    opacity: 0.3;
    margin-bottom: 16px;
    filter: drop-shadow(0 0 8px rgba(0, 255, 170, 0.3));
  }

  .empty-title {
    font-family: "JetBrains Mono", monospace;
    font-size: 14px;
    font-weight: 600;
    color: #e0e8f0;
    margin-bottom: 8px;
  }

  .empty-sub {
    font-size: 11px;
    color: #7088a0;
    line-height: 1.5;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 12px;
    font-size: 9px;
    color: #7088a0;
    font-family: "JetBrains Mono", monospace;
    border-top: 1px solid rgba(112, 136, 160, 0.08);
    background: rgba(8, 10, 18, 0.5);
  }

  .agent-count {
    color: #00ffaa;
    opacity: 0.6;
  }

  /* Invisible resize handles at window edges */
  :global(.resize-n), :global(.resize-s), :global(.resize-e), :global(.resize-w),
  :global(.resize-ne), :global(.resize-nw), :global(.resize-se), :global(.resize-sw) {
    position: fixed;
    z-index: 9999;
  }
  :global(.resize-n) {
    top: 0; left: 6px; right: 6px; height: 6px;
    cursor: n-resize;
  }
  :global(.resize-s) {
    bottom: 0; left: 6px; right: 6px; height: 6px;
    cursor: s-resize;
  }
  :global(.resize-e) {
    top: 6px; right: 0; bottom: 6px; width: 6px;
    cursor: e-resize;
  }
  :global(.resize-w) {
    top: 6px; left: 0; bottom: 6px; width: 6px;
    cursor: w-resize;
  }
  :global(.resize-ne) {
    top: 0; right: 0; width: 6px; height: 6px;
    cursor: ne-resize;
  }
  :global(.resize-nw) {
    top: 0; left: 0; width: 6px; height: 6px;
    cursor: nw-resize;
  }
  :global(.resize-se) {
    bottom: 0; right: 0; width: 12px; height: 12px;
    cursor: se-resize;
  }
  :global(.resize-sw) {
    bottom: 0; left: 0; width: 6px; height: 6px;
    cursor: sw-resize;
  }
</style>
