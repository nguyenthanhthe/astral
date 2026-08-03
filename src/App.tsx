import React, { useState, useEffect } from 'react';
import { Gamepad2, ShieldCheck, Play, Square, RefreshCw, Sparkles, Clock, Zap } from 'lucide-react';

const PRESET_GAMES = [
  { name: 'PLAYERUNKNOWN\'S BATTLEGROUNDS', exe: 'TslGame.exe', id: '356875221078245376' },
  { name: 'League of Legends', exe: 'League of Legends.exe', id: '1041071192534597652' },
  { name: 'Valorant', exe: 'VALORANT-Win64-Shipping.exe', id: '700136079562375258' },
  { name: 'Fortnite', exe: 'FortniteClient-Win64-Shipping.exe', id: '432980957394370560' },
  { name: 'Overwatch 2', exe: 'Overwatch.exe', id: '367827983903490050' },
];

export function App() {
  const [query, setQuery] = useState('');
  const [activeGame, setActiveGame] = useState<string | null>(null);
  const [timerMinutes, setTimerMinutes] = useState(15);
  const [secondsLeft, setSecondsLeft] = useState(15 * 60);
  const [isRunning, setIsRunning] = useState(false);

  useEffect(() => {
    let interval: any = null;
    if (isRunning && secondsLeft > 0) {
      interval = setInterval(() => {
        setSecondsLeft((prev) => prev - 1);
      }, 1000);
    } else if (secondsLeft === 0) {
      setIsRunning(false);
    }
    return () => clearInterval(interval);
  }, [isRunning, secondsLeft]);

  const handleStart = (gameName: string) => {
    setActiveGame(gameName);
    setSecondsLeft(timerMinutes * 60);
    setIsRunning(true);
  };

  const handleStop = () => {
    setIsRunning(false);
    setActiveGame(null);
  };

  const formatTime = (totalSec: number) => {
    const m = Math.floor(totalSec / 60);
    const s = totalSec % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  const progressPercent = ((timerMinutes * 60 - secondsLeft) / (timerMinutes * 60)) * 100;

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
          <span>Discord Direct IPC Active</span>
        </div>
      </header>

      <div className="main-grid">
        <div className="card">
          <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <Sparkles size={20} color="#38bdf8" /> Activity Spoofer
          </h2>
          
          <input
            type="text"
            className="input-field"
            placeholder="Enter custom executable or game name..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />

          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button className="btn-primary" onClick={() => handleStart(query || 'Custom Game.exe')} style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
              <Play size={16} /> Start Custom Game
            </button>
            {isRunning && (
              <button onClick={handleStop} style={{ padding: '0.75rem 1rem', background: '#ef4444', border: 'none', borderRadius: '8px', color: 'white', fontWeight: 600, cursor: 'pointer' }}>
                <Square size={16} /> Stop
              </button>
            )}
          </div>

          <div style={{ marginTop: '0.5rem' }}>
            <span style={{ fontSize: '0.85rem', color: 'var(--text-muted)', fontWeight: 600 }}>POPULAR QUEST PRESETS</span>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '0.5rem' }}>
              {PRESET_GAMES.map((game) => (
                <div 
                  key={game.id}
                  onClick={() => handleStart(game.name)}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '0.6rem 0.8rem',
                    background: activeGame === game.name ? 'rgba(56, 189, 248, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                    border: activeGame === game.name ? '1px solid #38bdf8' : '1px solid var(--border-color)',
                    borderRadius: '8px',
                    cursor: 'pointer',
                    transition: 'all 0.2s ease'
                  }}
                >
                  <div>
                    <div style={{ fontSize: '0.9rem', fontWeight: 600 }}>{game.name}</div>
                    <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>{game.exe}</div>
                  </div>
                  <Zap size={16} color={activeGame === game.name ? '#38bdf8' : '#9ca3af'} />
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="card" style={{ justifyContent: 'space-between' }}>
          <div>
            <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <Clock size={20} color="#a855f7" /> Tokio High-Precision Timer
            </h2>
            <p style={{ color: 'var(--text-muted)', fontSize: '0.85rem', marginTop: '0.25rem' }}>
              Sub-millisecond Rust async event loop with zero UI freezing.
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
              <span style={{ color: 'var(--text-muted)' }}>Active Target:</span>
              <span style={{ fontWeight: 600, color: activeGame ? '#38bdf8' : 'var(--text-muted)' }}>
                {activeGame ? activeGame : 'None'}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Discord Account:</span>
              <span style={{ fontWeight: 600 }}>telecom.no1</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>License:</span>
              <span style={{ color: '#22c55e', display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
                <ShieldCheck size={14} /> MIT Open Source
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
