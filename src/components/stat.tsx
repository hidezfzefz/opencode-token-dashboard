import { formatCompact, formatNumber } from "@/lib/format";

interface StatProps {
  label: string;
  value: number;
  meta?: string;
}

export function Stat({ label, value, meta }: StatProps) {
  return (
    <div className="card">
      <span className="stat-label">{label}</span>
      <strong className="stat-value" title={formatNumber(value)}>
        {formatCompact(value)}
      </strong>
      {meta ? <div className="stat-meta">{meta}</div> : null}
    </div>
  );
}
