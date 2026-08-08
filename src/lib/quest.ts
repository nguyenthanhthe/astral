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

/** Human-readable launch target for a quest (console/stream markers or .exe). */
export function questTargetLabel(exeName: string): string {
  if (exeName === CONSOLE_QUEST_MARKER) return 'Console (PS5 / Xbox)';
  if (exeName === STREAM_QUEST_MARKER) return 'Voice stream';
  return exeName;
}
