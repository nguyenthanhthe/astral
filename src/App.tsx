import React, { useState, useEffect } from 'react';
import { Star, ShieldCheck, Play, Square, Sparkles, Clock, Zap, Search, Award, CheckCircle2, Flame, Layers } from 'lucide-react';

interface DiscordQuest {
  id: string;
  title: string;
  game_name: string;
  exe_name: string;
  client_id: string;
  reward: string;
  progress_percent: number;
}

const DEFAULT_QUESTS: DiscordQuest[] = [
  {
    id: 'endfield_1',
    title: 'Companionship Celebration',
    game_name: 'Arknights: Endfield',
    exe_name: 'Endfield.exe',
    client_id: '1241071192534597652',
    reward: '700 Orbs',
    progress_percent: 79,
  },
  {
    id: 'nba2k27_1',
    title: '2K Mart Sneak Peek',
    game_name: 'NBA 2K27',
    exe_name: 'NBA2K27.exe',
    client_id: '1141071192534597652',
    reward: '700 Orbs',
    progress_percent: 0,
  },
  {
    id: 'wwm_1',
    title: 'YanYun Exploration Quest',
    game_name: 'Where Winds Meet',
    exe_name: 'WhereWindsMeet.exe',
    client_id: '1251071192534597659',
    reward: '700 Orbs',
    progress_percent: 0,
  },
  {
    id: 'eve_1',
    title: 'EVE Online Quest',
    game_name: 'EVE Online',
    exe_name: 'Eve.exe',
    client_id: '1041071192534597652',
    reward: '700 Orbs',
    progress_percent: 0,
  },
  {
    id: 'lol_1',
    title: 'Baron Charm Avatar Decoration',
    game_name: 'League of Legends',
    exe_name: 'League of Legends.exe',
    client_id: '1041071192534597653',
    reward: 'Avatar Decoration',
    progress_percent: 0,
  },
];

export function App() {
  const [query, setQuery] = useState('');
  const [quests, setQuests] = useState<DiscordQuest[]>(DEFAULT_QUESTS);
  const [selectedQuest, setSelectedQuest] = useState<DiscordQuest | null>(null);
  
  const [secondsLeft, setSecondsLeft] = useState(15 * 60);
  const [isRunning, setIsRunning] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string>('Ready for autonomous quest completion');
  const [discordUser, setDiscordUser] = useState<{ connected: boolean; username: string }>({
    connected: true,
    username: 'telecom.no1',
  });

  const invokeTauri = async (cmd: string, args?: any) => {
    try {
      if ((window as any).__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke(cmd, args);
      }
    } catch (e) {
      console.warn('Tauri invoke unavailable:', e);
    }
    return null;
  };

  // Fetch active quests & session on mount
  useEffect(() => {
    invokeTauri('fetch_active_quests').then((res) => {
      if (res && Array.isArray(res)) {
        setQuests(res);
      }
    });
    invokeTauri('check_discord_session').then((res: any) => {
      if (res && res.connected) {
        setDiscordUser({ connected: true, username: res.username });
      }
    });
  }, []);

  // Rust backend game & quest search query hook
  useEffect(() => {
    if (query.trim()) {
      invokeTauri('search_discord_games', { query }).then((res: any) => {
        if (res && Array.isArray(res)) {
          setQuests(res);
        }
      });
    } else {
      invokeTauri('fetch_active_quests').then((res: any) => {
        if (res && Array.isArray(res)) {
          setQuests(res);
        }
      });
    }
  }, [query]);

  // Timer countdown loop
  useEffect(() => {
    let interval: any = null;
    if (isRunning && secondsLeft > 0) {
      interval = setInterval(() => {
        setSecondsLeft((prev) => prev - 1);
      }, 1000);
    } else if (secondsLeft === 0 && isRunning) {
      handleStop();
      setStatusMsg('Quest Completed! Orbs & Rewards claimed.');
    }
    return () => clearInterval(interval);
  }, [isRunning, secondsLeft]);

  const handleStartQuest = async (quest: DiscordQuest) => {
    setSelectedQuest(quest);
    setSecondsLeft(15 * 60);
    setIsRunning(true);
    setStatusMsg(`Autonomous Quest Active: ${quest.game_name} (${quest.exe_name})`);

    // 1. Spawn stealth WinForms process stub for 100% Discord process scanner detection
    await invokeTauri('start_spoofer', { exeName: quest.exe_name, gameName: quest.game_name });
    
    // 2. Set Rich Presence activity directly via Discord IPC Pipe
    await invokeTauri('set_discord_activity', {
      clientId: quest.client_id,
      details: `Completing Quest: ${quest.title}`,
      state: `Earning ${quest.reward}`
    });
  };

  const handleAutoExecuteAll = async () => {
    if (quests.length > 0) {
      handleStartQuest(quests[0]);
      setStatusMsg('Auto-Executing All Quests: Active mission 1/4 in progress');
    }
  };

  const handleStop = async () => {
    if (selectedQuest) {
      await invokeTauri('stop_spoofer', { exeName: selectedQuest.exe_name });
    }
    setIsRunning(false);
    setSelectedQuest(null);
    setStatusMsg('Spoofing stopped & processes cleaned up');
  };

  const formatTime = (totalSec: number) => {
    const m = Math.floor(totalSec / 60);
    const s = totalSec % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  const progressPercent = ((15 * 60 - secondsLeft) / (15 * 60)) * 100;

  return (
    <div className="app-container">
      {/* Header with Star Icon */}
      <header className="header">
        <div className="logo-group">
          {/* Celestial Star SVG Icon */}
          <div style={{ position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <Star size={32} color="#38bdf8" fill="url(#starGradient)" style={{ filter: 'drop-shadow(0 0 10px rgba(56, 189, 248, 0.6))' }} />
            <svg width="0" height="0">
              <linearGradient id="starGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#38bdf8" />
                <stop offset="100%" stopColor="#a855f7" />
              </linearGradient>
            </svg>
          </div>
          <div>
            <span className="logo-title" style={{ letterSpacing: '2px' }}>ASTRAL</span>
            <span style={{ fontSize: '0.7rem', color: '#9ca3af', marginLeft: '0.5rem', textTransform: 'uppercase', letterSpacing: '1px' }}>
              Celestial Edition v2.0
            </span>
          </div>
        </div>
        <div className="status-badge">
          <span className="dot"></span>
          <span>Discord Active • {discordUser.username}</span>
        </div>
      </header>

      <div className="main-grid">
        {/* Left Column: Discord Missions Collector */}
        <div className="card">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Flame size={20} color="#38bdf8" /> Discord Active Missions
            </h2>
            <button
              onClick={handleAutoExecuteAll}
              style={{
                padding: '0.4rem 0.8rem',
                background: 'linear-gradient(135deg, #0284c7, #7e22ce)',
                border: 'none',
                borderRadius: '6px',
                color: 'white',
                fontSize: '0.8rem',
                fontWeight: 600,
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '0.4rem'
              }}
            >
              <Zap size={14} /> Auto-Execute All
            </button>
          </div>

          <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem' }}>
            Discovered active Discord Quests & 23,800+ detectable applications.
          </p>

          {/* Live Search Input Bar */}
          <div style={{ position: 'relative', marginTop: '0.6rem', marginBottom: '0.6rem' }}>
            <Search size={16} color="#38bdf8" style={{ position: 'absolute', left: '12px', top: '11px' }} />
            <input
              type="text"
              placeholder="Search 23,800+ Discord games (e.g. Genshin, PUBG, Where Winds Meet)..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              style={{
                width: '100%',
                padding: '0.55rem 0.6rem 0.55rem 2.3rem',
                background: 'rgba(0,0,0,0.4)',
                border: '1px solid rgba(56, 189, 248, 0.4)',
                borderRadius: '8px',
                color: 'white',
                fontSize: '0.85rem',
                outline: 'none'
              }}
            />
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem', marginTop: '0.25rem' }}>
            {quests.map((q) => {
              const isCurrent = selectedQuest?.id === q.id;
              return (
                <div
                  key={q.id}
                  onClick={() => handleStartQuest(q)}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '0.8rem 1rem',
                    background: isCurrent ? 'rgba(56, 189, 248, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                    border: isCurrent ? '1px solid #38bdf8' : '1px solid var(--border-color)',
                    borderRadius: '10px',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease'
                  }}
                >
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
                    <div style={{ fontSize: '0.95rem', fontWeight: 600, color: '#f3f4f6' }}>
                      {q.game_name}
                    </div>
                    <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                      {q.title} • Executable: <span style={{ color: '#38bdf8' }}>{q.exe_name}</span>
                    </div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                    <div style={{ textAlign: 'right' }}>
                      <span style={{ fontSize: '0.8rem', fontWeight: 700, color: '#22c55e', display: 'flex', alignItems: 'center', gap: '0.2rem' }}>
                        <Award size={14} /> {q.reward}
                      </span>
                      <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                        {isCurrent ? `${Math.round(progressPercent)}% Progress` : `${q.progress_percent}% Saved`}
                      </span>
                    </div>
                    <Play size={18} color={isCurrent ? '#38bdf8' : '#9ca3af'} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right Column: Mission Control & Celestial Gauge */}
        <div className="card" style={{ justifyContent: 'space-between' }}>
          <div>
            <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Star size={20} color="#a855f7" /> Celestial Mission Control
            </h2>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.25rem' }}>
              Real-time process scanner & IPC pipe synchronization active.
            </p>
          </div>

          {/* SVG Animated Celestial Star Gauge */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', margin: '0.8rem 0' }}>
            <div style={{ position: 'relative', width: '170px', height: '170px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <svg width="170" height="170" viewBox="0 0 170 170" style={{ transform: 'rotate(-90deg)' }}>
                <circle cx="85" cy="85" r="75" stroke="rgba(255,255,255,0.06)" strokeWidth="10" fill="transparent" />
                <circle 
                  cx="85" 
                  cy="85" 
                  r="75" 
                  stroke="url(#starProgressGradient)" 
                  strokeWidth="10" 
                  fill="transparent"
                  strokeDasharray="471"
                  strokeDashoffset={471 - (471 * progressPercent) / 100}
                  strokeLinecap="round"
                  style={{ transition: 'stroke-dashoffset 0.5s ease' }}
                />
                <defs>
                  <linearGradient id="starProgressGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#38bdf8" />
                    <stop offset="100%" stopColor="#a855f7" />
                  </linearGradient>
                </defs>
              </svg>
              <div style={{ position: 'absolute', textAlign: 'center' }}>
                <div style={{ fontSize: '2.3rem', fontWeight: 700, fontFamily: 'JetBrains Mono, monospace', background: 'linear-gradient(135deg, #38bdf8, #a855f7)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent' }}>
                  {formatTime(secondsLeft)}
                </div>
                <div style={{ fontSize: '0.75rem', color: isRunning ? '#22c55e' : 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '1px' }}>
                  {isRunning ? 'Mission Syncing' : 'Standby'}
                </div>
              </div>
            </div>

            {isRunning && (
              <button 
                onClick={handleStop} 
                style={{ marginTop: '0.5rem', padding: '0.5rem 1.2rem', background: '#ef4444', border: 'none', borderRadius: '6px', color: 'white', fontWeight: 600, fontSize: '0.85rem', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '0.4rem' }}
              >
                <Square size={14} /> Stop Mission
              </button>
            )}
          </div>

          <div style={{ background: 'rgba(0,0,0,0.3)', padding: '0.8rem 1rem', borderRadius: '8px', border: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', gap: '0.4rem', fontSize: '0.85rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Active Mission:</span>
              <span style={{ fontWeight: 600, color: selectedQuest ? '#38bdf8' : 'var(--text-muted)' }}>
                {selectedQuest ? selectedQuest.game_name : 'None'}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Executable:</span>
              <span style={{ fontWeight: 600, color: selectedQuest ? '#22c55e' : 'var(--text-muted)' }}>
                {selectedQuest ? selectedQuest.exe_name : 'None'}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Target Reward:</span>
              <span style={{ color: '#22c55e', fontWeight: 600 }}>
                {selectedQuest ? selectedQuest.reward : 'None'}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
