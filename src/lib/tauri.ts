import { invoke } from '@tauri-apps/api/core';
import type {
  CustomQuestInput,
  DiscordQuest,
  DiscordStatus,
  SessionStarted,
} from './quest';

/** Check whether Discord IPC is reachable and return the logged-in user. */
export function checkDiscordSession(): Promise<DiscordStatus> {
  return invoke<DiscordStatus>('check_discord_session');
}

/** Fetch the curated quest presets plus the user's mirrored quests. */
export function fetchActiveQuests(): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('fetch_active_quests');
}

/** Search the Discord game backend for a matching quest. */
export function searchDiscordGames(query: string): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('search_discord_games', { query });
}

/** List the quests mirrored from the user's Discord client. */
export function listCustomQuests(): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('list_custom_quests');
}

/** Validate, persist and return a new mirrored quest. */
export function saveCustomQuest(input: CustomQuestInput): Promise<DiscordQuest> {
  return invoke<DiscordQuest>('save_custom_quest', { input });
}

/** Delete a mirrored quest by id. */
export function deleteCustomQuest(id: string): Promise<void> {
  return invoke<void>('delete_custom_quest', { id });
}

/** Force a network refresh of the detectable-games catalog. */
export function refreshCatalog(): Promise<void> {
  return invoke<void>('refresh_catalog');
}

/**
 * Start a quest session on the backend (the engine drives progress).
 * `demo = true` runs it display-only: no spoofer process, no Discord IPC.
 */
export function startSession(quest: DiscordQuest, demo = false): Promise<void> {
  return invoke<void>('start_session', { quest, demo });
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

export interface UpdateInfo {
  latest_version: string;
  current_version: string;
  is_update_available: boolean;
  url: string;
}

/** Query the latest astral GitHub release and report whether we're outdated. */
export function checkForUpdate(): Promise<UpdateInfo> {
  return invoke<UpdateInfo>('check_for_update');
}

export interface Settings {
  memory_trim_on_start: boolean;
  /** Run sessions display-only by default (persisted on disk by the backend). */
  demo_mode: boolean;
}

export interface SettingsPatch {
  memory_trim_on_start?: boolean;
  demo_mode?: boolean;
}

/** Read the current settings. */
export function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

/** Apply an additive settings patch. */
export function setSettings(patch: SettingsPatch): Promise<Settings> {
  return invoke<Settings>('set_settings', { patch });
}
