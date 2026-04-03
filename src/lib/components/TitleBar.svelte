<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  function minimize() {
    getCurrentWindow().minimize();
  }

  function close() {
    getCurrentWindow().close();
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-left" data-tauri-drag-region>
    <span class="logo">&#x25C8;</span>
    <span class="title">AGENTVIEW</span>
  </div>
  <div class="titlebar-right">
    <button class="titlebar-btn minimize" onclick={minimize}>&#x2014;</button>
    <button class="titlebar-btn close" onclick={close}>&#x2715;</button>
  </div>
</div>

<style>
  .titlebar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 36px;
    padding: 0 12px;
    background: rgba(8, 10, 18, 0.95);
    border-bottom: 1px solid rgba(0, 255, 170, 0.15);
    user-select: none;
    -webkit-user-select: none;
    cursor: grab;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo {
    color: #00ffaa;
    font-size: 16px;
    filter: drop-shadow(0 0 4px rgba(0, 255, 170, 0.5));
  }

  .title {
    font-family: "Orbitron", "JetBrains Mono", monospace;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 3px;
    color: #00ffaa;
    text-shadow: 0 0 8px rgba(0, 255, 170, 0.4);
  }

  .titlebar-right {
    display: flex;
    gap: 4px;
  }

  .titlebar-btn {
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: #7088a0;
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .titlebar-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #e0e8f0;
  }

  .titlebar-btn.close:hover {
    background: rgba(255, 60, 60, 0.3);
    color: #ff6060;
  }
</style>
