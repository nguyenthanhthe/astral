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
  onStop: () => void;
}

export function SessionPanel({
  running,
  quest,
  progress,
  secondsLeft,
  message,
  onStop,
}: SessionPanelProps) {
  const caption = running ? `${formatTime(secondsLeft)} left` : 'Standby';

  return (
    <div className="session-status">
      <ProgressRing value={progress} caption={caption} running={running} />
      <p className="session-message" data-state={running ? 'running' : 'idle'} role="status">
        {message}
      </p>

      <dl className="session-details">
        <div className="session-detail">
          <dt className="session-detail__label">Active mission</dt>
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
