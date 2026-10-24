import { useState } from 'react';
import { Plus, X } from 'lucide-react';
import { type CustomQuestKind, type DiscordQuest } from '../lib/quest';
import { deleteCustomQuest, saveCustomQuest } from '../lib/tauri';
import { Button } from './Button';

interface QuestBuilderProps {
  /** Mirrored quests shown for management (delete). */
  customQuests: DiscordQuest[];
  onChanged: () => void;
  onClose: () => void;
}

const KIND_OPTIONS: { value: CustomQuestKind; label: string }[] = [
  { value: 'exe', label: 'Game (.exe)' },
  { value: 'video', label: 'Video' },
  { value: 'console', label: 'Console (PS5 / Xbox)' },
  { value: 'stream', label: 'Voice stream' },
];

const DEFAULT_DURATION: Record<CustomQuestKind, number> = {
  exe: 15 * 60,
  video: 30,
  console: 30,
  stream: 30,
};

/** Form for mirroring a quest the user sees in their own Discord client. */
export function QuestBuilder({ customQuests, onChanged, onClose }: QuestBuilderProps) {
  const [title, setTitle] = useState('');
  const [kind, setKind] = useState<CustomQuestKind>('video');
  const [gameName, setGameName] = useState('');
  const [exeName, setExeName] = useState('');
  const [orbs, setOrbs] = useState(700);
  const [durationSec, setDurationSec] = useState(DEFAULT_DURATION.video);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const pickKind = (next: CustomQuestKind) => {
    setKind(next);
    setDurationSec(DEFAULT_DURATION[next]);
  };

  const submit = async () => {
    setSaving(true);
    setError(null);
    try {
      await saveCustomQuest({
        title: title.trim(),
        game_name: gameName.trim(),
        kind,
        exe_name: kind === 'exe' ? exeName.trim() || null : null,
        client_id: null,
        reward: `${Math.max(0, Math.floor(orbs))} Orbs`,
        duration_secs: Math.max(0, Math.floor(durationSec)),
      });
      setTitle('');
      setGameName('');
      setExeName('');
      onChanged();
    } catch (e) {
      // AppError serializes as { code, message }; message is user-safe.
      const message =
        typeof e === 'object' && e !== null && 'message' in e
          ? String((e as { message: unknown }).message)
          : "Couldn't save the quest.";
      setError(message);
    } finally {
      setSaving(false);
    }
  };

  const remove = async (id: string) => {
    try {
      await deleteCustomQuest(id);
      onChanged();
    } catch {
      setError("Couldn't delete the quest.");
    }
  };

  return (
    <div className="quest-builder" role="form" aria-label="Add a quest">
      <div className="quest-builder__header">
        <h3 className="quest-builder__title">Add a quest</h3>
        <Button variant="ghost" size="sm" onClick={onClose} aria-label="Close quest builder">
          <X size={14} aria-hidden="true" />
        </Button>
      </div>
      <p className="quest-builder__hint">
        Recreate a quest you see in your Discord client. Saved on this device only — your
        account is never accessed.
      </p>

      <label className="qb-field">
        <span className="qb-field__label">Quest title</span>
        <input
          className="qb-input"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="e.g. Watch the launch trailer"
          maxLength={120}
        />
      </label>

      <div className="qb-row">
        <label className="qb-field">
          <span className="qb-field__label">Type</span>
          <select
            className="qb-input"
            value={kind}
            onChange={(e) => pickKind(e.target.value as CustomQuestKind)}
          >
            {KIND_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </label>

        <label className="qb-field">
          <span className="qb-field__label">Duration (s)</span>
          <input
            className="qb-input"
            type="number"
            min={10}
            max={86400}
            value={durationSec}
            onChange={(e) => setDurationSec(Number(e.target.value))}
          />
        </label>

        <label className="qb-field">
          <span className="qb-field__label">Reward (Orbs)</span>
          <input
            className="qb-input"
            type="number"
            min={1}
            max={100000}
            value={orbs}
            onChange={(e) => setOrbs(Number(e.target.value))}
          />
        </label>
      </div>

      <label className="qb-field">
        <span className="qb-field__label">Game name</span>
        <input
          className="qb-input"
          value={gameName}
          onChange={(e) => setGameName(e.target.value)}
          placeholder="e.g. Arknights: Endfield"
          maxLength={80}
        />
      </label>

      {kind === 'exe' && (
        <label className="qb-field">
          <span className="qb-field__label">Executable name</span>
          <input
            className="qb-input"
            value={exeName}
            onChange={(e) => setExeName(e.target.value)}
            placeholder="e.g. Endfield.exe"
            maxLength={120}
          />
        </label>
      )}

      {error && (
        <p className="quest-builder__error" role="alert">
          {error}
        </p>
      )}

      <div className="quest-builder__actions">
        <Button variant="primary" size="sm" onClick={submit} disabled={saving}>
          <Plus size={12} aria-hidden="true" />
          {saving ? 'Saving…' : 'Add quest'}
        </Button>
      </div>

      {customQuests.length > 0 && (
        <div className="quest-builder__managed">
          <h4 className="quest-builder__managed-title">Mirrored quests</h4>
          <ul className="quest-builder__list">
            {customQuests.map((quest) => (
              <li key={quest.id} className="quest-builder__item">
                <span className="quest-builder__item-name">{quest.title}</span>
                <span className="quest-builder__item-meta">
                  {quest.game_name} · {quest.duration_secs ?? '—'}s
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => remove(quest.id)}
                  aria-label={`Delete ${quest.title}`}
                >
                  <X size={12} aria-hidden="true" />
                </Button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
