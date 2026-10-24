export interface DiscordQuest {
  id: string;
  title: string;
  game_name: string;
  exe_name: string;
  client_id: string;
  reward: string;
  progress_percent: number;
  /** True when the executable is backed by Discord's detectable-game catalog. */
  catalog_verified: boolean;
  /** Real quest length (mirrored quests); undefined → kind-based default. */
  duration_secs?: number | null;
}

/** Mirrored-quest kind. */
export type CustomQuestKind = 'exe' | 'console' | 'stream' | 'video';

/** Input for `save_custom_quest` (backend validates and assigns the id). */
export interface CustomQuestInput {
  title: string;
  game_name: string;
  kind: CustomQuestKind;
  exe_name?: string | null;
  client_id?: string | null;
  reward: string;
  duration_secs: number;
}

export interface DiscordStatus {
  connected: boolean;
  username: string;
  user_id: string;
}

/** `session://started` / `get_session_status` payload. */
export interface SessionStarted {
  session_id: string;
  quest_id: string;
  game_name: string;
  exe_name: string;
  target_sec: number;
  initial_percent: number;
  /** True when the session is display-only (no spoofer, no Discord IPC). */
  demo?: boolean;
}

/** `session://progress` payload (engine, every second). */
export interface SessionProgress {
  session_id: string;
  percent: number;
  elapsed_sec: number;
  remaining_sec: number;
}

/** `session://finished` payload. */
export interface SessionFinished {
  session_id: string;
  quest_id: string;
}

/** `session://stopped` payload. */
export interface SessionStopped {
  session_id: string;
  reason: 'USER' | 'ERROR';
  /** User-safe failure reason; present only when reason is ERROR. */
  message?: string | null;
}

export const VIDEO_QUEST_DURATION_SEC = 30;
export const GAME_QUEST_DURATION_SEC = 15 * 60;

/** Format seconds as MM:SS. */
export function formatTime(totalSec: number): string {
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

const CONSOLE_QUEST_MARKER = '[Console Quest]';
const STREAM_QUEST_MARKER = '[Stream Quest]';
const VIDEO_QUEST_MARKER = '[Video Quest]';

/** Human-readable launch target for a quest (console/stream/video markers or .exe). */
export function questTargetLabel(exeName: string): string {
  if (exeName === CONSOLE_QUEST_MARKER) return 'Console (PS5 / Xbox)';
  if (exeName === STREAM_QUEST_MARKER) return 'Voice stream';
  if (exeName === VIDEO_QUEST_MARKER) return 'Quest video';
  return exeName;
}

/** Extract the orb amount from a display reward ("700 Orbs" → 700). */
export function orbCountFromReward(reward: string): number | null {
  const match = reward.trim().match(/^(\d+)\s+orbs?$/i);
  return match ? Number(match[1]) : null;
}
