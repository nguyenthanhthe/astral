import { Square } from 'lucide-react';
import { type DiscordQuest, formatTime, questTargetLabel } from '../lib/quest';
import { Button } from './Button';
import { ProgressRing } from './ProgressRing';

interface SessionPanelProps {
  running: boolean;
  quest: DiscordQuest | null;
  progress: number;
  secondsLeft: number;
  message: string;
  /** True when the running session is display-only (demo mode). */
  demo?: boolean;
  onStop: () => void;
}

/** Quiet constellation anchoring the idle session state. */
function Constellation() {
  return (
    <svg
      className="constellation"
      width="120"
      height="64"
      viewBox="0 0 120 64"
      fill="none"
      aria-hidden="true"
    >
      <path
        d="M8 52 L34 30 L58 40 L84 12 L112 22"
        stroke="currentColor"
        strokeWidth="1"
        strokeDasharray="2 4"
      />
      {[
        [8, 52],
        [34, 30],
        [58, 40],
        [84, 12],
        [112, 22],
      ].map(([x, y], i) => (
        <circle
          key={i}
          cx={x}
          cy={y}
          r={i === 3 ? 2.5 : 1.5}
          fill={i === 3 ? 'var(--brand)' : 'currentColor'}
        />
      ))}
    </svg>
  );
}

export function SessionPanel({
  running,
  quest,
  progress,
  secondsLeft,
  message,
  demo = false,
  onStop,
}: SessionPanelProps) {
  const caption = running ? `${formatTime(secondsLeft)} left` : 'Standby';
  const idle = !running && !quest;

  return (
    <div className="session-status">
      {idle && <Constellation />}
      <ProgressRing value={progress} caption={caption} running={running} />
      <p className="session-message" data-state={running ? 'running' : 'idle'} role="status">
        {message}
        {demo && running && <span className="session-demo-badge">DEMO</span>}
      </p>

      <dl className="session-details">
        <div className="session-detail">
          <dt className="session-detail__label">Mission</dt>
          <dd className="session-detail__value">{quest?.game_name ?? '—'}</dd>
        </div>
        <div className="session-detail">
          <dt className="session-detail__label">Target</dt>
          <dd className="session-detail__value">
            {quest ? <code>{questTargetLabel(quest.exe_name)}</code> : '—'}
          </dd>
        </div>
        <div className="session-detail">
          <dt className="session-detail__label">Reward</dt>
          <dd className="session-detail__value">{quest?.reward ?? '—'}</dd>
        </div>
      </dl>

      {running && (
        <div className="session-actions">
          <Button variant="danger" onClick={onStop}>
            <Square size={14} aria-hidden="true" />
            Stop session
          </Button>
        </div>
      )}
    </div>
  );
}
