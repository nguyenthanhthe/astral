import React, { useState } from 'react';
import { Gamepad2, Activity, ShieldCheck, Play } from 'lucide-react';

export function App() {
  const [query, setQuery] = useState('');
  const [activeGame, setActiveGame] = useState<string | null>(null);

  const handleLaunch = () => {
    if (query.trim()) {
      setActiveGame(query.trim());
    }
  };

  return (
    <div className="app-container">
      <header className="header">
        <div className="logo-group">
          <Gamepad2 size={28} color="#38bdf8" />
          <span className="logo-title">ASTRAL</span>
        </div>
        <div className="status-badge">
          <span className="dot"></span>
          <span>Discord IPC Active</span>
        </div>
      </header>

      <div className="main-grid">
        <div className="card">
          <h2 className="card-title">Activity Spoofer</h2>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.9rem' }}>
            Select or enter a game executable to spoof active presence on Discord.
          </p>
          <input
            type="text"
            className="input-field"
            placeholder="Search game name (e.g. PUBG, Valorant, LoL)..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <button className="btn-primary" onClick={handleLaunch} style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.5rem' }}>
            <Play size={18} />
            Start Quest Simulation
          </button>
        </div>

        <div className="card">
          <h2 className="card-title">Active Session Info</h2>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', fontSize: '0.9rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Logged User:</span>
              <span style={{ fontWeight: 600 }}>telecom.no1</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>Active Spoofed Activity:</span>
              <span style={{ color: activeGame ? '#38bdf8' : 'var(--text-muted)' }}>
                {activeGame ? activeGame : 'None'}
              </span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <span style={{ color: 'var(--text-muted)' }}>License:</span>
              <span style={{ color: '#22c55e', display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
                <ShieldCheck size={16} /> MIT Open Source
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
