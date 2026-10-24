import { describe, it, expect } from 'vitest';
import {
  formatTime,
  orbCountFromReward,
  questTargetLabel,
  type DiscordQuest,
  type SessionStopped,
} from './quest';

const exeQuest: DiscordQuest = {
  id: '1',
  title: 'Game Quest',
  game_name: 'Some Game',
  exe_name: 'Game.exe',
  client_id: '1',
  reward: '700 Orbs',
  progress_percent: 0,
  catalog_verified: true,
};

describe('formatTime', () => {
  it('formats MM:SS with padding', () => {
    expect(formatTime(0)).toBe('00:00');
    expect(formatTime(5)).toBe('00:05');
    expect(formatTime(900)).toBe('15:00');
    expect(formatTime(61)).toBe('01:01');
  });
});

describe('questTargetLabel', () => {
  it('humanizes console and stream markers', () => {
    expect(questTargetLabel('[Console Quest]')).toBe('Console (PS5 / Xbox)');
    expect(questTargetLabel('[Stream Quest]')).toBe('Voice stream');
  });

  it('humanizes the video marker', () => {
    expect(questTargetLabel('[Video Quest]')).toBe('Quest video');
  });

  it('passes through executable names', () => {
    expect(questTargetLabel('Endfield.exe')).toBe('Endfield.exe');
  });
});

describe('orbCountFromReward', () => {
  it('parses orb display strings', () => {
    expect(orbCountFromReward('700 Orbs')).toBe(700);
    expect(orbCountFromReward('50 orbs')).toBe(50);
    expect(orbCountFromReward('  1200 ORBS ')).toBe(1200);
  });

  it('returns null for non-orb rewards', () => {
    expect(orbCountFromReward('1 Gem')).toBeNull();
    expect(orbCountFromReward('Orbs')).toBeNull();
    expect(orbCountFromReward('')).toBeNull();
  });
});

// The session engine owns progress math (Rust side, `session://progress`
// events). These are compile-time guards that the wire contract types still
// carry the fields the backend serializes.
describe('session wire contract', () => {
  it('DiscordQuest keeps the fields the engine needs', () => {
    expect(exeQuest.exe_name).toBe('Game.exe');
    expect(typeof exeQuest.progress_percent).toBe('number');
  });

  it('SessionStopped reason is USER or ERROR', () => {
    const stopped: SessionStopped = { session_id: 's1', reason: 'USER' };
    expect(['USER', 'ERROR']).toContain(stopped.reason);
  });
});
