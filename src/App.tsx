import { useCallback, useEffect, useRef, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { listen } from '@tauri-apps/api/event';
import { Activity, AlertCircle, Flag, Play, Wand2 } from 'lucide-react';
import {
  checkDiscordSession,
  checkForUpdate,
  fetchActiveQuests,
  getSettings,
  getSessionStatus,
  listCustomQuests,
  optimizeRam,
  searchDiscordGames,
  setSettings,
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
import { orbCountFromReward } from './lib/quest';
import { AppHeader, type ConnectionState, type UpdateState } from './components/AppHeader';
import { Button } from './components/Button';
import { QuestBuilder } from './components/QuestBuilder';
import { QuestList } from './components/QuestList';
import { RewardOverlay } from './components/RewardOverlay';
import { SearchInput } from './components/SearchInput';
import { SessionPanel } from './components/SessionPanel';
import { Switch } from './components/Switch';

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

  // Demo mode: sessions run purely locally — the progress ring and reward
  // overlay are display-only, with no spoofer process and no Discord traffic.
  // Persisted by the backend (`settings.json`), not webview storage.
  const [demoMode, setDemoMode] = useState(false);
  const [runningDemo, setRunningDemo] = useState(false);
  const [builderOpen, setBuilderOpen] = useState(false);
  const [customQuests, setCustomQuests] = useState<DiscordQuest[]>([]);
  const [rewardOverlay, setRewardOverlay] = useState<{
    orbs: number | null;
    label: string;
  } | null>(null);

  // Refs mirror the state the once-registered engine listeners read, so the
  // finish handler never sees a stale closure after a re-render.
  const runningDemoRef = useRef(false);
  const selectedQuestRef = useRef<DiscordQuest | null>(null);
  useEffect(() => {
    selectedQuestRef.current = selectedQuest;
  }, [selectedQuest]);

  const toggleDemoMode = useCallback((next: boolean) => {
    setDemoMode(next);
    setSettings({ demo_mode: next }).catch(() => undefined);
  }, []);

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

  const loadCustomQuests = useCallback(() => {
    listCustomQuests()
      .then(setCustomQuests)
      .catch(() => undefined);
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
    getSettings()
      .then((s) => setDemoMode(s.demo_mode))
      .catch(() => undefined);
    // Re-hydrate an engine-owned session after a reload.
    getSessionStatus()
      .then((status) => {
        if (!status) return;
        setIsRunning(true);
        setRunningDemo(status.demo ?? false);
        runningDemoRef.current = status.demo ?? false;
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
        const demo = s.demo ?? false;
        runningDemoRef.current = demo;
        setRunningDemo(demo);
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
        if (runningDemoRef.current) {
          const quest = selectedQuestRef.current;
          setRewardOverlay({
            orbs: orbCountFromReward(quest?.reward ?? ''),
            label: quest?.reward ?? '',
          });
          setSessionMessage('Demo session complete — simulated locally.');
        } else {
          setSessionMessage('Session complete (simulated) — confirm the reward in Discord.');
        }
        setSelectedQuest(null);
        setProgress(100);
        setSecondsLeft(0);
      }),
      listen<SessionStopped>('session://stopped', (event) => {
        const failed = event.payload.reason === 'ERROR';
        setIsRunning(false);
        setSelectedQuest(null);
        if (failed) {
          // Surface the real reason (e.g. the Windows-only spoofer) instead
          // of leaving the panel looking silently frozen.
          setSessionError(
            event.payload.message ?? 'The session stopped due to an error.',
          );
        }
        setSessionMessage(failed ? 'Session stopped due to an error.' : 'Session stopped.');
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

  const startSession = useCallback(
    async (quest: DiscordQuest) => {
      setSelectedQuest(quest);
      setSessionError(null);
      setIsRunning(true);
      setProgress(quest.progress_percent || 0);
      setSessionMessage(`Running ${quest.game_name}${demoMode ? ' (demo)' : ''}`);

      try {
        await invokeStartSession(quest, demoMode);
      } catch (e) {
        setIsRunning(false);
        setSelectedQuest(null);
        setSessionMessage(`Couldn't start ${quest.game_name}.`);
        // AppError serializes as { code, message }; prefer the user-safe text.
        setSessionError(
          typeof e === 'object' && e !== null && 'message' in e
            ? String((e as { message: unknown }).message)
            : String(e),
        );
      }
    },
    [demoMode],
  );

  const runFirstQuest = useCallback(() => {
    if (quests.length > 0) startSession(quests[0]);
  }, [quests, startSession]);

  const closeRewardOverlay = useCallback(() => setRewardOverlay(null), []);

  const openBuilder = useCallback(() => {
    setBuilderOpen(true);
    loadCustomQuests();
  }, [loadCustomQuests]);

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
          <span>{sessionError}</span>
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
            <div className="panel-header__actions">
              <Button
                variant={builderOpen ? 'secondary' : 'ghost'}
                size="sm"
                onClick={builderOpen ? () => setBuilderOpen(false) : openBuilder}
              >
                <Wand2 size={12} aria-hidden="true" />
                Mirror quest
              </Button>
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
          </div>

          <SearchInput value={query} onChange={setQuery} />

          {builderOpen && (
            <QuestBuilder
              customQuests={customQuests}
              onChanged={() => {
                loadCustomQuests();
                loadQuests();
              }}
              onClose={() => setBuilderOpen(false)}
            />
          )}

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
          <div className="panel-header">
            <h2 id="session-heading" className="panel-title">
              <Activity size={14} aria-hidden="true" />
              Session
            </h2>
            <Switch
              checked={demoMode}
              onChange={toggleDemoMode}
              label="Demo"
              title="Run sessions display-only — no spoofer, no Discord IPC"
            />
          </div>
          <SessionPanel
            running={isRunning}
            quest={selectedQuest}
            progress={progress}
            secondsLeft={secondsLeft}
            message={sessionMessage}
            demo={runningDemo}
            onStop={stopSession}
          />
        </aside>
      </div>

      <RewardOverlay
        open={rewardOverlay !== null}
        orbs={rewardOverlay?.orbs ?? null}
        rewardLabel={rewardOverlay?.label ?? ''}
        onClose={closeRewardOverlay}
      />
    </div>
  );
}

export default App;
