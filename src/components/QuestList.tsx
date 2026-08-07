import { Award } from 'lucide-react';
import {
  type DiscordQuest,
  questTargetLabel,
} from '../lib/quest';
import { Button } from './Button';

interface QuestListProps {
  quests: DiscordQuest[];
  runningQuestId: string | null;
  liveProgress: number;
  loading: boolean;
  error: boolean;
  query: string;
  onSelect: (quest: DiscordQuest) => void;
  onRetry: () => void;
}

function QuestRow({
  quest,
  isRunning,
  progress,
  onSelect,
}: {
  quest: DiscordQuest;
  isRunning: boolean;
  progress: number;
  onSelect: (quest: DiscordQuest) => void;
}) {
  return (
    <li>
      <button
        type="button"
        className="quest-row"
        aria-current={isRunning ? 'true' : undefined}
        onClick={() => onSelect(quest)}
      >
        <span className="quest-row__main">
          <span className="quest-row__name">{quest.game_name}</span>
          <span className="quest-row__meta">
            {quest.title} · Target <code>{questTargetLabel(quest.exe_name)}</code>
          </span>
        </span>
        <span className="quest-row__side">
          <span className="quest-row__reward">
            <Award size={14} aria-hidden="true" />
            {quest.reward}
          </span>
          <span className="quest-row__progress-label">
            {isRunning ? `${progress}% running` : `${quest.progress_percent}% saved`}
          </span>
        </span>
        {progress > 0 && (
          <span className="quest-row__bar" aria-hidden="true">
            <span className="quest-row__bar-fill" style={{ width: `${progress}%` }} />
          </span>
        )}
      </button>
    </li>
  );
}

export function QuestList({
  quests,
  runningQuestId,
  liveProgress,
  loading,
  error,
  query,
  onSelect,
  onRetry,
}: QuestListProps) {
  if (loading) {
    return (
      <ul className="quest-list" aria-busy="true" aria-label="Loading quests">
        {Array.from({ length: 4 }).map((_, i) => (
          <li key={i}>
            <div className="skeleton-row" />
          </li>
        ))}
      </ul>
    );
  }

  if (error) {
    return (
      <div className="state-block" role="alert">
        <p className="state-block__title">Couldn't load quests</p>
        <p className="state-block__hint">The Quest backend didn't respond. Try again.</p>
        <Button variant="secondary" size="sm" onClick={onRetry}>
          Retry
        </Button>
      </div>
    );
  }

  if (quests.length === 0) {
    return (
      <div className="state-block">
        <p className="state-block__title">
          {query.trim() ? 'No matching quests' : 'No active quests right now'}
        </p>
        <p className="state-block__hint">
          {query.trim()
            ? 'Try a different search term.'
            : 'Quests will appear here when Discord has active promotions.'}
        </p>
      </div>
    );
  }

  return (
    <ul className="quest-list">
      {quests.map((quest) => (
        <QuestRow
          key={quest.id}
          quest={quest}
          isRunning={runningQuestId === quest.id}
          progress={runningQuestId === quest.id ? liveProgress : quest.progress_percent}
          onSelect={onSelect}
        />
      ))}
    </ul>
  );
}
