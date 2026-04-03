import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { AgentInfo } from "../types/agent";

export const agents = writable<AgentInfo[]>([]);
export const lastUpdated = writable<Date>(new Date());
export const isLoading = writable<boolean>(false);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function fetchAgents() {
  isLoading.set(true);
  try {
    const result = await invoke<AgentInfo[]>("get_agents");
    agents.set(result);
    lastUpdated.set(new Date());
  } catch (e) {
    console.error("Failed to fetch agents:", e);
  } finally {
    isLoading.set(false);
  }
}

export function startPolling(intervalMs = 5000) {
  // Fetch immediately
  fetchAgents();

  // Then poll
  if (pollInterval) clearInterval(pollInterval);
  pollInterval = setInterval(fetchAgents, intervalMs);
}

export function stopPolling() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}
