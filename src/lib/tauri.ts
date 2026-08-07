import { invoke } from '@tauri-apps/api/core';
import type { DiscordQuest, DiscordStatus } from './quest';

/** Check whether Discord IPC is reachable and return the logged-in user. */
export function checkDiscordSession(): Promise<DiscordStatus> {
  return invoke<DiscordStatus>('check_discord_session');
}

/** Fetch the hardcoded active Discord quest list. */
export function fetchActiveQuests(): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('fetch_active_quests');
}

/** Search the 23,800+ Discord game backend. */
export function searchDiscordGames(query: string): Promise<DiscordQuest[]> {
  return invoke<DiscordQuest[]>('search_discord_games', { query });
}

/** Spoof a non-EXE (console/stream) quest via Discord IPC. */
export function spoofNonExeQuest(
  questType: string,
  clientId: string,
  gameName: string,
  durationSeconds: number,
): Promise<string> {
  return invoke<string>('spoof_non_exe_quest', { questType, clientId, gameName, durationSeconds });
}

/** Set Discord Rich Presence activity with a start/end progress window. */
export function setDiscordActivity(
  clientId: string,
  details: string,
  state: string,
  durationSeconds: number,
): Promise<string> {
  return invoke<string>('set_discord_activity', { clientId, details, state, durationSeconds });
}

/** Launch spoofer processes for an EXE quest. */
export function startSpoofer(exeName: string, gameName?: string): Promise<string> {
  return invoke<string>('start_spoofer', { exeName, gameName });
}

/** Stop spoofer processes and clean up staged executables. */
export function stopSpoofer(exeName: string): Promise<string> {
  return invoke<string>('stop_spoofer', { exeName });
}

/** Trim unmapped WebView2 memory pages (Windows only). */
export function optimizeRam(): Promise<string> {
  return invoke<string>('optimize_ram');
}
