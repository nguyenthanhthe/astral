interface ProgressRingProps {
  value: number;
  caption: string;
  running: boolean;
}

const SIZE = 128;
const STROKE = 6;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ProgressRing({ value, caption, running }: ProgressRingProps) {
  const clamped = Math.max(0, Math.min(100, value));
  const offset = CIRCUMFERENCE * (1 - clamped / 100);

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
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={offset}
        />
      </svg>
      <div className="ring__center">
        <span className="ring__value">{Math.round(clamped)}%</span>
        <span className="ring__caption">{caption}</span>
      </div>
    </div>
  );
}
