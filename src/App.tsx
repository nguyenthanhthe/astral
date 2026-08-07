import { useCallback, useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { listen } from '@tauri-apps/api/event';
import { Activity, AlertCircle, Flag, Play } from 'lucide-react';
import {
  checkDiscordSession,
  checkForUpdate,
  fetchActiveQuests,
  getSessionStatus,
  optimizeRam,
  searchDiscordGames,
  startSession as invokeStartSession,
  stopSession as invokeStopSession,
} from './lib/tauri';
import type {
  DiscordQuest,
  DiscordStatus,
  SessionFinished,
  SessionProgress,
  SessionStarted,
  SessionStopped,
} from './lib/quest';
import { AppHeader, type ConnectionState, type UpdateState } from './components/AppHeader';
import { Button } from './components/Button';
import { QuestList } from './components/QuestList';
import { SearchInput } from './components/SearchInput';
import { SessionPanel } from './components/SessionPanel';

const DEFAULT_QUESTS: DiscordQuest[] = [];

const DISCONNECTED_STATUS: DiscordStatus = {
  connected: false,
  username: 'Disconnected',
  user_id: '',
};

export function App() {
  const [query, setQuery] = useState('');
  const [quests, setQuests] = useState<DiscordQuest[]>(DEFAULT_QUESTS);
  const [questsLoading, setQuestsLoading] = useState(true);
  const [questsError, setQuestsError] = useState(false);
  const [selectedQuest, setSelectedQuest] = useState<DiscordQuest | null>(null);

  const [progress, setProgress] = useState(0);
  const [secondsLeft, setSecondsLeft] = useState(15 * 60);
  const [isRunning, setIsRunning] = useState(false);
  const [sessionMessage, setSessionMessage] = useState('Select a quest to start a session.');
  const [sessionError, setSessionError] = useState<string | null>(null);
  const [discordUser, setDiscordUser] = useState<DiscordStatus>(DISCONNECTED_STATUS);
  const [connection, setConnection] = useState<ConnectionState>('checking');
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const [updateState, setUpdateState] = useState<UpdateState>('idle');
  const [updateLatest, setUpdateLatest] = useState<string | undefined>(undefined);
  const [updateUrl, setUpdateUrl] = useState<string | undefined>(undefined);

  const runUpdateCheck = useCallback(() => {
    setUpdateState('checking');
    checkForUpdate()
      .then((info) => {
        if (info.is_update_available) {
          setUpdateLatest(info.latest_version);
          setUpdateUrl(info.url);
          setUpdateState('available');
        } else {
          setUpdateState('uptodate');
        }
      })
      .catch(() => setUpdateState('error'));
  }, []);

  const loadQuests = useCallback(() => {
    setQuestsLoading(true);
    fetchActiveQuests()
      .then((qs) => {
        setQuests(qs);
        setQuestsError(false);
      })
      .catch(() => setQuestsError(true))
      .finally(() => setQuestsLoading(false));
  }, []);

  useEffect(() => {
    loadQuests();
    optimizeRam().catch(() => undefined);
    getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion(null));
    checkDiscordSession()
      .then((user) => {
        setDiscordUser(user);
        setConnection(user.connected ? 'connected' : 'disconnected');
      })
      .catch(() => setConnection('disconnected'));
    runUpdateCheck();
    // Re-hydrate an engine-owned session after a reload.
    getSessionStatus()
      .then((status) => {
        if (!status) return;
        setIsRunning(true);
        setSessionMessage(`Running ${status.game_name}`);
      })
      .catch(() => undefined);
  }, [loadQuests, runUpdateCheck]);

  // Live Discord connection pill: the backend connection task pushes
  // `discord://status` on connect/disconnect/reconnect (Phase 1, §7.2).
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<DiscordStatus>('discord://status', (event) => {
      setDiscordUser(event.payload);
      setConnection(event.payload.connected ? 'connected' : 'disconnected');
    })
      .then((un) => {
        if (cancelled) un();
        else unlisten = un;
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  // Session engine events (Phase 3): progress no longer lives in the FE timer.
  useEffect(() => {
    let cancelled = false;
    const unlisteners: (() => void)[] = [];
    const subs: Promise<() => void>[] = [
      listen<SessionStarted>('session://started', (event) => {
        const s = event.payload;
        setIsRunning(true);
        setProgress(s.initial_percent);
        setSecondsLeft(s.target_sec);
        setSessionMessage(`Running ${s.game_name}`);
      }),
      listen<SessionProgress>('session://progress', (event) => {
        setProgress(event.payload.percent);
        setSecondsLeft(event.payload.remaining_sec);
      }),
      listen<SessionFinished>('session://finished', () => {
        setIsRunning(false);
        setSelectedQuest(null);
        setProgress(100);
        setSecondsLeft(0);
        setSessionMessage('Quest complete.');
      }),
      listen<SessionStopped>('session://stopped', (event) => {
        setIsRunning(false);
        setSelectedQuest(null);
        setSessionMessage(
          event.payload.reason === 'ERROR'
            ? 'Session stopped due to an error.'
            : 'Session stopped.',
        );
      }),
    ];
    subs.forEach((sub) =>
      sub.then((un) => {
        if (cancelled) un();
        else unlisteners.push(un);
      }),
    );
    return () => {
      cancelled = true;
      unlisteners.forEach((un) => un());
    };
  }, []);

  useEffect(() => {
    const handler = setTimeout(() => {
      setQuestsLoading(true);
      const request = query.trim() ? searchDiscordGames(query) : fetchActiveQuests();
      request
        .then((qs) => {
          setQuests(qs);
          setQuestsError(false);
        })
        .catch(() => setQuestsError(true))
        .finally(() => setQuestsLoading(false));
    }, 350);

    return () => clearTimeout(handler);
  }, [query]);

  const stopSession = useCallback(async () => {
    setSessionMessage('Stopping session…');
    try {
      await invokeStopSession();
    } catch {
      // The engine clears itself; the stopped event settles the UI.
      setIsRunning(false);
      setSelectedQuest(null);
      setSessionMessage('Session stopped.');
    }
  }, []);

  const startSession = useCallback(async (quest: DiscordQuest) => {
    setSelectedQuest(quest);
    setSessionError(null);
    setIsRunning(true);
    setProgress(quest.progress_percent || 0);
    setSessionMessage(`Running ${quest.game_name}`);

    try {
      await invokeStartSession(quest);
    } catch (e) {
      setIsRunning(false);
      setSelectedQuest(null);
      setSessionMessage(`Couldn't start ${quest.game_name}.`);
      setSessionError(String(e));
    }
  }, []);

  const runFirstQuest = useCallback(() => {
    if (quests.length > 0) startSession(quests[0]);
  }, [quests, startSession]);

  return (
    <div className="app-shell">
      <AppHeader
        version={appVersion}
        connection={connection}
        username={discordUser.username}
        updateState={updateState}
        latestVersion={updateLatest}
        releaseUrl={updateUrl}
        onCheckForUpdate={runUpdateCheck}
      />

      {sessionError && (
        <div className="error-banner" role="alert">
          <AlertCircle size={16} aria-hidden="true" />
          <span>Failed to start the session.</span>
          <span className="error-banner__actions">
            <Button variant="ghost" size="sm" onClick={() => setSessionError(null)}>
              Dismiss
            </Button>
          </span>
        </div>
      )}

      <div className="workspace">
        <section className="panel" aria-labelledby="quests-heading">
          <div className="panel-header">
            <h2 id="quests-heading" className="panel-title">
              <Flag size={14} aria-hidden="true" />
              Quests
            </h2>
            <Button
              variant="secondary"
              size="sm"
              onClick={runFirstQuest}
              disabled={quests.length === 0 || isRunning}
            >
              <Play size={12} aria-hidden="true" />
              Start first quest
            </Button>
          </div>

          <SearchInput value={query} onChange={setQuery} />

          <div className="panel-scroll">
            <QuestList
              quests={quests}
              runningQuestId={selectedQuest?.id ?? null}
              liveProgress={progress}
              loading={questsLoading}
              error={questsError}
              query={query}
              onSelect={startSession}
              onRetry={loadQuests}
            />
          </div>
        </section>

        <aside className="panel" aria-labelledby="session-heading">
          <h2 id="session-heading" className="panel-title">
            <Activity size={14} aria-hidden="true" />
            Session
          </h2>
          <SessionPanel
            running={isRunning}
            quest={selectedQuest}
            progress={progress}
            secondsLeft={secondsLeft}
            message={sessionMessage}
            onStop={stopSession}
          />
        </aside>
      </div>
    </div>
  );
}

export default App;
