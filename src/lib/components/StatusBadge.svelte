<script lang="ts">
  import type { AgentStatus } from "../types/agent";

  interface Props {
    status: AgentStatus;
  }

  let { status }: Props = $props();

  const labels: Record<AgentStatus, string> = {
    running: "RUNNING",
    waiting: "WAITING",
    idle: "IDLE",
    dead: "DEAD",
  };
</script>

<div class="badge badge-{status}">
  <span class="dot"></span>
  <span class="label">{labels[status]}</span>
</div>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 1px;
    font-family: "JetBrains Mono", monospace;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .badge-running {
    background: rgba(0, 255, 170, 0.1);
    color: #00ffaa;
  }
  .badge-running .dot {
    background: #00ffaa;
    box-shadow: 0 0 6px #00ffaa;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .badge-waiting {
    background: rgba(255, 190, 60, 0.1);
    color: #ffbe3c;
  }
  .badge-waiting .dot {
    background: #ffbe3c;
    box-shadow: 0 0 6px #ffbe3c;
    animation: pulse 2s ease-in-out infinite;
  }

  .badge-idle {
    background: rgba(112, 136, 160, 0.1);
    color: #7088a0;
  }
  .badge-idle .dot {
    background: #7088a0;
  }

  .badge-dead {
    background: rgba(255, 60, 60, 0.1);
    color: #ff3c3c;
  }
  .badge-dead .dot {
    background: #ff3c3c;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50% { opacity: 0.5; transform: scale(0.8); }
  }
</style>
