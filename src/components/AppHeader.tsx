import { Github, RefreshCw, Star } from 'lucide-react';
import { open } from '@tauri-apps/plugin-shell';
import { StatusPill, type ConnectionState } from './StatusPill';

export type { ConnectionState };

export type UpdateState = 'idle' | 'checking' | 'uptodate' | 'available' | 'error';

const REPO_URL = 'https://github.com/nguyenthanhthe/astral';

interface AppHeaderProps {
  version: string | null;
  connection: ConnectionState;
  username?: string;
  updateState: UpdateState;
  latestVersion?: string;
  releaseUrl?: string;
  onCheckForUpdate: () => void;
}

function openExternal(url: string) {
  open(url).catch(() => undefined);
}

export function AppHeader({
  version,
  connection,
  username,
  updateState,
  latestVersion,
  releaseUrl,
  onCheckForUpdate,
}: AppHeaderProps) {
  const updateLabel =
    updateState === 'checking'
      ? 'Checking…'
      : updateState === 'uptodate'
        ? 'Up to date'
        : updateState === 'error'
          ? 'Check failed'
          : 'Check for updates';

  return (
    <header className="app-header">
      <div className="brand-group">
        <span className="brand-mark" aria-hidden="true">
          <Star size={18} strokeWidth={1.5} fill="currentColor" />
        </span>
        <div className="brand-title">
          <span className="brand-name">ASTRAL</span>
          {version && <span className="brand-version">v{version}</span>}
        </div>
      </div>
      <div className="header-actions">
        {updateState === 'available' && latestVersion && releaseUrl ? (
          <button
            type="button"
            className="update-pill update-pill--available"
            onClick={() => openExternal(releaseUrl)}
            title={`astral ${latestVersion} is available — open the release page`}
          >
            <span className="update-pill__dot" aria-hidden="true" />
            v{latestVersion.replace(/^v/, '')} available
          </button>
        ) : (
          <button
            type="button"
            className="update-pill"
            onClick={onCheckForUpdate}
            disabled={updateState === 'checking'}
            title="Check for a newer astral release"
          >
            <RefreshCw
              size={12}
              aria-hidden="true"
              className={updateState === 'checking' ? 'spin' : undefined}
            />
            {updateLabel}
          </button>
        )}
        <a
          className="github-link"
          href={REPO_URL}
          onClick={(e) => {
            e.preventDefault();
            openExternal(REPO_URL);
          }}
          aria-label="Astral source code on GitHub"
          title="Astral on GitHub"
        >
          <Github size={18} aria-hidden="true" />
        </a>
        <StatusPill state={connection} username={username} />
      </div>
    </header>
  );
}
