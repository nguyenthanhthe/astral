import { Star } from 'lucide-react';
import { StatusPill, type ConnectionState } from './StatusPill';

export type { ConnectionState };

interface AppHeaderProps {
  version: string | null;
  connection: ConnectionState;
  username?: string;
}

export function AppHeader({ version, connection, username }: AppHeaderProps) {
  return (
    <header className="app-header">
      <div className="brand-group">
        <span className="brand-mark" aria-hidden="true">
          <Star size={18} strokeWidth={1.5} />
        </span>
        <div className="brand-title">
          <span className="brand-name">ASTRAL</span>
          {version && <span className="brand-version">v{version}</span>}
        </div>
      </div>
      <StatusPill state={connection} username={username} />
    </header>
  );
}
