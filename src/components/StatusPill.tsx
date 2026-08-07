export type ConnectionState = 'checking' | 'connected' | 'disconnected';

interface StatusPillProps {
  state: ConnectionState;
  username?: string;
}

const LABELS: Record<ConnectionState, string> = {
  checking: 'Checking Discord…',
  connected: 'Connected',
  disconnected: 'Not connected',
};

export function StatusPill({ state, username }: StatusPillProps) {
  const label = state === 'connected' && username ? `Connected · ${username}` : LABELS[state];

  return (
    <span className={`status-pill status-pill--${state}`}>
      <span className="status-dot" aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}
