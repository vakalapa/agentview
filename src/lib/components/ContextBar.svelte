<script lang="ts">
  import { formatTokenCount } from "../utils/format";

  interface Props {
    percent: number | null;
    totalTokens: number;
    contextWindow: number;
  }

  let { percent, totalTokens, contextWindow }: Props = $props();

  let barColor = $derived(
    percent === null
      ? "#7088a0"
      : percent < 50
        ? "#00ffaa"
        : percent < 80
          ? "#ffbe3c"
          : "#ff3c3c"
  );

  let clampedPercent = $derived(
    percent === null ? 0 : Math.min(percent, 100)
  );
</script>

<div class="context-bar">
  <div class="bar-track">
    <div
      class="bar-fill"
      style="width: {clampedPercent}%; background: {barColor}; box-shadow: 0 0 8px {barColor}40;"
    ></div>
  </div>
  <div class="bar-label">
    {#if percent !== null}
      <span class="percent">{percent.toFixed(1)}%</span>
      <span class="detail">{formatTokenCount(totalTokens)} / {formatTokenCount(contextWindow)}</span>
    {:else}
      <span class="percent na">N/A</span>
    {/if}
  </div>
</div>

<style>
  .context-bar {
    width: 100%;
  }

  .bar-track {
    height: 4px;
    background: rgba(112, 136, 160, 0.15);
    border-radius: 2px;
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.6s ease, background 0.3s ease;
  }

  .bar-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 3px;
    font-size: 9px;
    font-family: "JetBrains Mono", monospace;
  }

  .percent {
    color: #e0e8f0;
    font-weight: 600;
  }

  .percent.na {
    color: #7088a0;
  }

  .detail {
    color: #7088a0;
  }
</style>
