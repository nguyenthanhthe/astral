import { useId } from 'react';

interface ProgressRingProps {
  value: number;
  caption: string;
  running: boolean;
}

const SIZE = 148;
const STROKE = 6;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/**
 * The Orbit — Astral's signature progress ring. The arc carries a
 * cyan→violet gradient and a comet-head dot at its tip while running.
 */
export function ProgressRing({ value, caption, running }: ProgressRingProps) {
  const gradientId = useId();
  const clamped = Math.max(0, Math.min(100, value));
  const offset = CIRCUMFERENCE * (1 - clamped / 100);
  const theta = (clamped / 100) * 2 * Math.PI - Math.PI / 2;
  const headX = SIZE / 2 + RADIUS * Math.cos(theta);
  const headY = SIZE / 2 + RADIUS * Math.sin(theta);

  return (
    <div
      className="ring"
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(clamped)}
      aria-valuetext={running ? `${Math.round(clamped)} percent, ${caption}` : caption}
    >
      <svg
        width={SIZE}
        height={SIZE}
        viewBox={`0 0 ${SIZE} ${SIZE}`}
        aria-hidden="true"
      >
        <defs>
          <linearGradient id={gradientId} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="var(--brand)" />
            <stop offset="100%" stopColor="var(--nebula)" />
          </linearGradient>
        </defs>
        <circle
          className="ring__track"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          fill="none"
          strokeWidth={STROKE}
        />
        <circle
          className="ring__bar"
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          fill="none"
          strokeWidth={STROKE}
          stroke={`url(#${gradientId})`}
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
        />
        {clamped > 0 && clamped < 100 && (
          <>
            {running && (
              <circle
                className="ring__head-glow"
                cx={headX}
                cy={headY}
                r={7}
                fill="var(--brand)"
              />
            )}
            <circle
              className="ring__head"
              cx={headX}
              cy={headY}
              r={3.5}
              fill={running ? 'var(--brand)' : 'var(--text-muted)'}
            />
          </>
        )}
      </svg>
      <div className="ring__center">
        <span className="ring__value">{Math.round(clamped)}%</span>
        <span className="ring__caption">{caption}</span>
      </div>
    </div>
  );
}
