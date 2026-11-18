interface ShareBarProps {
  value: number;
  max: number;
  label: string;
}

export function ShareBar({ value, max, label }: ShareBarProps) {
  const width = max > 0 ? Math.max(2, Math.round((value / max) * 100)) : 0;
  return (
    <div
      className="share-track"
      role="img"
      aria-label={`${label}: ${Math.round((value / max) * 100) || 0}% of the largest entry`}
    >
      <div className="share-fill" style={{ width: `${width}%` }} />
    </div>
  );
}
