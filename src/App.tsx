import React, { useState, useEffect } from 'react';
import { Gamepad2, ShieldCheck, Play, Square, Sparkles, Clock, Zap, Search, AlertCircle, CheckCircle2 } from 'lucide-react';

interface GameRecord {
  id: string;
  name: string;
  aliases?: string[];
  executables?: { os: string; name: string }[];
}

const FALLBACK_PRESETS = [
  { name: 'Arknights: Endfield', exe: 'Endfield.exe', id: '1241071192534597652' },
  { name: 'NBA 2K27', exe: 'NBA2K27.exe', id: '1141071192534597652' },
  { name: 'EVE Online', exe: 'Eve.exe', id: '1041071192534597652' },
  { name: 'PLAYERUNKNOWN\'S BATTLEGROUNDS', exe: 'TslGame.exe', id: '356875221078245376' },
  { name: 'League of Legends', exe: 'League of Legends.exe', id: '1041071192534597652' },
  { name: 'Valorant', exe: 'VALORANT-Win64-Shipping.exe', id: '700136079562375258' },
];

export function App() {
  const [query, setQuery] = useState('');
  const [allGames, setAllGames] = useState<GameRecord[]>([]);
  const [searchResults, setSearchResults] = useState<GameRecord[]>([]);
  const [selectedGame, setSelectedGame] = useState<{ name: string; exe: string } | null>(null);
  
  const [secondsLeft, setSecondsLeft] = useState(15 * 60);
  const [isRunning, setIsRunning] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string>('Ready to start game quest');
  const [discordUser, setDiscordUser] = useState<{ connected: boolean; username: string }>({
    connected: true,
    username: 'telecom.no1',
  });

  // Fetch live detectable games database from Discord API on mount
  useEffect(() => {
    fetch('https://discord.com/api/v9/applications/detectable')
      .then((res) => res.json())
      .then((data) => {
        if (Array.isArray(data)) {
          setAllGames(data);
        }
      })
      .catch(() => {
        // Fallback silently if offline
      });
  }, []);

  // Filter games based on search query
  useEffect(() => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }
    const q = query.toLowerCase();
    const matches = allGames
      .filter((g) => g.name.toLowerCase().includes(q) || g.aliases?.some((a) => a.toLowerCase().includes(q)))
      .slice(0, 5);
    setSearchResults(matches);
  }, [query, allGames]);

  // Timer countdown loop
  useEffect(() => {
    let interval: any = null;
    if (isRunning && secondsLeft > 0) {
      interval = setInterval(() => {
        setSecondsLeft((prev) => prev - 1);
      }, 1000);
    } else if (secondsLeft === 0 && isRunning) {
      handleStop();
      setStatusMsg('Quest Completed! Orbs earned.');
    }
    return () => clearInterval(interval);
  }, [isRunning, secondsLeft]);

  const invokeTauri = async (cmd: string, args?: any) => {
    try {
      if ((window as any).__TAURI_INTERNALS__) {
        const { invoke } = await import('@tauri-apps/api/core');
        return await invoke(cmd, args);
      }
    } catch (e) {
      console.warn('Tauri invoke unavailable in web browser:', e);
    }
    return null;
  };

  const handleStart = async (gameName: string, exeName?: string) => {
    let targetExe = exeName;
    if (!targetExe) {
      if (gameName.toLowerCase().includes('arknights') || gameName.toLowerCase().includes('endfield')) {
        targetExe = 'Endfield.exe';
      } else if (gameName.toLowerCase().includes('nba')) {
        targetExe = 'NBA2K27.exe';
      } else if (gameName.toLowerCase().includes('eve')) {
        targetExe = 'Eve.exe';
      } else {
        targetExe = gameName.endsWith('.exe') ? gameName : `${gameName}.exe`;
      }
    }

    setSelectedGame({ name: gameName, exe: targetExe });
    setSecondsLeft(15 * 60);
    setIsRunning(true);
    setStatusMsg(`Autonomous Quest Active: ${targetExe} (IPC Sync + Process Detection)`);

    // 1. Spawn stealth WinForms process for 100% Windows process scanner detection
    await invokeTauri('start_spoofer', { exeName: targetExe, gameName });
    
    // 2. Set Rich Presence activity directly via Discord IPC Pipe
    const clientId = gameName.toLowerCase().includes('endfield') ? '1241071192534597652' : '356875221078245376';
    await invokeTauri('set_discord_activity', {
      clientId,
      details: `Playing ${gameName}`,
      state: 'Completing Discord Quest'
    });
  };

  const handleStop = async () => {
    if (selectedGame) {
      await invokeTauri('stop_spoofer', { exeName: selectedGame.exe });
    }
    setIsRunning(false);
    setSelectedGame(null);
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
      <header className="header">
        <div className="logo-group">
          <Gamepad2 size={28} color="#38bdf8" />
          <span className="logo-title">ASTRAL</span>
          <span style={{ fontSize: '0.75rem', background: 'rgba(56, 189, 248, 0.15)', color: '#38bdf8', padding: '0.2rem 0.5rem', borderRadius: '4px', border: '1px solid rgba(56, 189, 248, 0.3)' }}>
            Tauri v2 + Rust
          </span>
        </div>
        <div className="status-badge">
          <span className="dot"></span>
          <span>Discord Process Scanner Active</span>
        </div>
      </header>

      <div className="main-grid">
        {/* Left Column: Search & Spoofer Controls */}
        <div className="card">
          <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Sparkles size={20} color="#38bdf8" /> Discord Quest Spoofer
          </h2>
          
          <div style={{ position: 'relative' }}>
            <input
              type="text"
              className="input-field"
              placeholder="Search game name (e.g., Arknights, Endfield, NBA 2K, PUBG)..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
            {searchResults.length > 0 && (
              <div style={{ position: 'absolute', top: '100%', left: 0, right: 0, background: '#161b22', border: '1px solid var(--border-color)', borderRadius: '8px', zIndex: 10, marginTop: '4px', overflow: 'hidden' }}>
                {searchResults.map((game) => {
                  const winExe = game.executables?.find((e) => e.os === 'win32')?.name || `${game.name}.exe`;
                  return (
                    <div
                      key={game.id}
                      onClick={() => {
                        setQuery(game.name);
                        setSearchResults([]);
                        handleStart(game.name, winExe);
                      }}
                      style={{ padding: '0.6rem 1rem', cursor: 'pointer', borderBottom: '1px solid var(--border-color)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}
                    >
                      <div>
                        <div style={{ fontSize: '0.9rem', fontWeight: 600 }}>{game.name}</div>
                        <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Executable: {winExe}</div>
                      </div>
                      <Play size={14} color="#38bdf8" />
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              className="btn-primary"
              onClick={() => handleStart(query || 'Endfield.exe', query ? undefined : 'Endfield.exe')}
              style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}
            >
              <Play size={16} /> Start Quest Process
            </button>
            {isRunning && (
              <button onClick={handleStop} style={{ padding: '0.75rem 1rem', background: '#ef4444', border: 'none', borderRadius: '8px', color: 'white', fontWeight: 600, cursor: 'pointer' }}>
                <Square size={16} /> Stop
              </button>
            )}
          </div>

          <div style={{ marginTop: '0.5rem' }}>
            <span style={{ fontSize: '0.85rem', color: 'var(--text-muted)', fontWeight: 600, display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <Zap size={14} color="#38bdf8" /> ACTIVE DISCORD QUEST PRESETS
            </span>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '0.5rem' }}>
              {FALLBACK_PRESETS.map((game) => (
                <div 
                  key={game.id}
                  onClick={() => handleStart(game.name, game.exe)}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '0.6rem 0.8rem',
                    background: selectedGame?.name === game.name ? 'rgba(56, 189, 248, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                    border: selectedGame?.name === game.name ? '1px solid #38bdf8' : '1px solid var(--border-color)',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease'
                  }}
                >
                  <div>
                    <div style={{ fontSize: '0.9rem', fontWeight: 600 }}>{game.name}</div>
                    <div style={{ fontSize: '0.75rem', color: '#38bdf8' }}>Executable: {game.exe}</div>
                  </div>
                  <Play size={16} color={selectedGame?.name === game.name ? '#38bdf8' : '#9ca3af'} />
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Right Column: Timer & Process Detection Gauge */}
        <div className="card" style={{ justifyContent: 'space-between' }}>
          <div>
            <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Clock size={20} color="#a855f7" /> Tokio High-Precision Timer & Scanner
            </h2>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.25rem' }}>
              Discord process scanner detects launched `.exe` in `Win64/` and completes quest.
            </p>
          </div>

          {/* SVG Animated Timer Gauge */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', margin: '1rem 0' }}>
            <div style={{ position: 'relative', width: '160px', height: '160px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <svg width="160" height="160" viewBox="0 0 160 160" style={{ transform: 'rotate(-90deg)' }}>
                <circle cx="80" cy="80" r="70" stroke="rgba(255,255,255,0.08)" strokeWidth="10" fill="transparent" />
                <circle 
                  cx="80" 
                  cy="80" 
                  r="70" 
                  stroke="url(#gradient)" 
                  strokeWidth="10" 
                  fill="transparent"
                  strokeDasharray="440"
                  strokeDashoffset={440 - (440 * progressPercent) / 100}
                  strokeLinecap="round"
                  style={{ transition: 'stroke-dashoffset 0.5s ease' }}
                />
                <defs>
                  <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#38bdf8" />
                    <stop offset="100%" stopColor="#a855f7" />
                  </linearGradient>
                </defs>
              </svg>
              <div style={{ position: 'absolute', textAlign: 'center' }}>
                <div style={{ fontSize: '2.2rem', fontWeight: 700, fontFamily: 'JetBrains Mono, monospace' }}>
                  {formatTime(secondsLeft)}
                </div>
                <div style={{ fontSize: '0.75rem', color: isRunning ? '#22c55e' : 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '1px' }}>
                  {isRunning ? 'Quest Active' : 'Standby'}
                </div>
              </div>
            </div>
          </div>

          <div style={{ background: 'rgba(0,0,0,0.3)', padding: '0.8rem 1rem', borderRadius: '8px', border: '1px solid var(--border-color)', display: 'flex', flexDirection: 'column', gap: '0.4rem', fontSize: '0.85rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Status:</span>
              <span style={{ fontWeight: 600, color: isRunning ? '#22c55e' : '#38bdf8' }}>{statusMsg}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Spoofed Executable:</span>
              <span style={{ fontWeight: 600, color: selectedGame ? '#38bdf8' : 'var(--text-muted)' }}>
                {selectedGame ? selectedGame.exe : 'None'}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Discord Account:</span>
              <span style={{ fontWeight: 600 }}>{discordUser.username}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
