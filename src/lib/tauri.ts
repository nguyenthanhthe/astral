import { invoke } from '@tauri-apps/api/core';
import type { DiscordQuest, DiscordStatus, SessionStarted } from './quest';

/** Check whether Discord IPC is reachable and return the logged-in user. */
export function checkDiscordSession(): Promise<DiscordStatus> {
  return invoke<DiscordStatus>('check_discord_session');
}

/** Fetch the hardcoded active Discord quest list. */
export function fetchActiveQuests(): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('fetch_active_quests');
}

/** Search the Discord game backend for a matching quest. */
export function searchDiscordGames(query: string): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('search_discord_games', { query });
}

/** Force a network refresh of the detectable-games catalog. */
export function refreshCatalog(): Promise<void> {
  return invoke<void>('refresh_catalog');
}

/** Start a quest session on the backend (the engine drives progress). */
export function startSession(quest: DiscordQuest): Promise<void> {
  return invoke<void>('start_session', { quest });
}

/** Stop the running session. */
export function stopSession(): Promise<void> {
  return invoke<void>('stop_session');
}

/** Current session state, or null when idle (UI re-hydration). */
export function getSessionStatus(): Promise<SessionStarted | null> {
  return invoke<SessionStarted | null>('get_session_status');
}

/** Trim unmapped WebView2 memory pages (Windows only). */
export function optimizeRam(): Promise<string> {
  return invoke<string>('optimize_ram');
}

export interface Settings {
  memory_trim_on_start: boolean;
}

export interface SettingsPatch {
  memory_trim_on_start?: boolean;
}

/** Read the current settings. */
export function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

/** Apply an additive settings patch. */
export function setSettings(patch: SettingsPatch): Promise<Settings> {
  return invoke<Settings>('set_settings', { patch });
}
