import { useEffect } from 'react';
import { Sparkles } from 'lucide-react';
import { Button } from './Button';

interface RewardOverlayProps {
  open: boolean;
  /** Orb amount parsed from the quest reward; null renders the reward label. */
  orbs: number | null;
  rewardLabel: string;
  onClose: () => void;
}

const AUTO_DISMISS_MS = 6000;

/**
 * Display-only reward celebration for demo sessions. Purely visual — the
 * caption says so, and nothing here touches Discord.
 */
export function RewardOverlay({ open, orbs, rewardLabel, onClose }: RewardOverlayProps) {
  useEffect(() => {
    if (!open) return;
    const timer = setTimeout(onClose, AUTO_DISMISS_MS);
    return () => clearTimeout(timer);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      className="reward-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Demo reward"
      onClick={onClose}
    >
      <div className="reward-overlay__card" onClick={(e) => e.stopPropagation()}>
        <span className="reward-overlay__orb reward-overlay__orb--1" aria-hidden="true" />
        <span className="reward-overlay__orb reward-overlay__orb--2" aria-hidden="true" />
        <span className="reward-overlay__orb reward-overlay__orb--3" aria-hidden="true" />
        <Sparkles size={28} aria-hidden="true" className="reward-overlay__icon" />
        <p className="reward-overlay__amount">
          {orbs !== null ? `+${orbs.toLocaleString()} Orbs` : `+${rewardLabel}`}
        </p>
        <p className="reward-overlay__note">
          Demo session complete — simulated locally. No real Orbs were earned.
        </p>
        <Button variant="secondary" size="sm" onClick={onClose}>
          Close
        </Button>
      </div>
    </div>
  );
}
