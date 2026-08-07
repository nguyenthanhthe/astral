import { describe, it, expect } from 'vitest';
import {
  isNonExeQuest,
  targetDurationSec,
  remainingSec,
  currentProgress,
  formatTime,
  type DiscordQuest,
} from './quest';

const exeQuest: DiscordQuest = {
  id: '1',
  title: 'Game Quest',
  game_name: 'Some Game',
  exe_name: 'Game.exe',
  client_id: '1',
  reward: '700 Orbs',
  progress_percent: 0,
};

describe('isNonExeQuest', () => {
  it('detects console/stream quests', () => {
    expect(isNonExeQuest({ exe_name: '[Console Quest]' })).toBe(true);
    expect(isNonExeQuest({ exe_name: '[Stream Quest]' })).toBe(true);
  });

  it('treats exe quests as regular', () => {
    expect(isNonExeQuest(exeQuest)).toBe(false);
  });
});

describe('targetDurationSec', () => {
  it('uses 30s for video quests', () => {
    expect(targetDurationSec({ exe_name: 'video.mp4' })).toBe(30);
    expect(targetDurationSec({ exe_name: 'WatchVideo.exe' })).toBe(30);
  });

  it('uses 15 minutes for game quests', () => {
    expect(targetDurationSec(exeQuest)).toBe(15 * 60);
  });
});

describe('remainingSec', () => {
  it('computes remaining from saved progress', () => {
    expect(remainingSec(900, 0)).toBe(900);
    expect(remainingSec(900, 50)).toBe(450);
    expect(remainingSec(900, 100)).toBe(1); // never zero
  });

  it('clamps to at least 1 second', () => {
    expect(remainingSec(30, 99)).toBe(1);
  });
});

describe('currentProgress', () => {
  it('starts at initial progress and reaches 100 at the end', () => {
    expect(currentProgress(0, 900, 900)).toBe(0);
    expect(currentProgress(0, 900, 0)).toBe(100);
    expect(currentProgress(50, 900, 450)).toBe(75);
  });

  it('never exceeds 100', () => {
    expect(currentProgress(0, 900, -100)).toBe(100);
  });

  it('handles zero-duration input without NaN', () => {
    expect(currentProgress(50, 0, 0)).toBe(0);
  });
});

describe('formatTime', () => {
  it('formats MM:SS with padding', () => {
    expect(formatTime(0)).toBe('00:00');
    expect(formatTime(5)).toBe('00:05');
    expect(formatTime(900)).toBe('15:00');
    expect(formatTime(61)).toBe('01:01');
  });
});
