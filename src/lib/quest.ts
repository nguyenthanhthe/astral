export interface DiscordQuest {
  id: string;
  title: string;
  game_name: string;
  exe_name: string;
  client_id: string;
  reward: string;
  progress_percent: number;
}

export interface DiscordStatus {
  connected: boolean;
  username: string;
  user_id: string;
}

export const VIDEO_QUEST_DURATION_SEC = 30;
export const GAME_QUEST_DURATION_SEC = 15 * 60;

/** True for console/stream quests that carry no `.exe` and use IPC spoofing. */
export function isNonExeQuest(quest: Pick<DiscordQuest, 'exe_name'>): boolean {
  return quest.exe_name.startsWith('[');
}

/** Target duration in seconds: 30s for video quests, 15 minutes for games. */
export function targetDurationSec(quest: Pick<DiscordQuest, 'exe_name'>): number {
  return quest.exe_name.toLowerCase().includes('video')
    ? VIDEO_QUEST_DURATION_SEC
    : GAME_QUEST_DURATION_SEC;
}

/** Remaining seconds given saved progress (0-100). */
export function remainingSec(reqSec: number, startProgress: number): number {
  return Math.max(1, Math.round(reqSec * (1 - startProgress / 100)));
}

/** Live progress 0-100 based on elapsed time from saved progress. */
export function currentProgress(
  initialProgress: number,
  totalRequiredSec: number,
  secondsLeft: number,
): number {
  if (totalRequiredSec <= 0) return 0;
  const elapsedRatio = (totalRequiredSec - secondsLeft) / totalRequiredSec;
  return Math.min(100, Math.round(initialProgress + elapsedRatio * (100 - initialProgress)));
}

/** Format seconds as MM:SS. */
export function formatTime(totalSec: number): string {
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}
