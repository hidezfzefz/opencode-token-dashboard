import Link from "next/link";
import { RefreshButton } from "@/components/refresh-button";
import { Stat } from "@/components/stat";
import { UsageChart } from "@/components/usage-chart";
import { formatCompact, formatNumber, formatPercent } from "@/lib/format";
import { getUsage } from "@/lib/usage";

export const instant = false;

export default async function OverviewPage({
  searchParams,
}: {
  searchParams: Promise<{ range?: string }>;
}) {
  const { range } = await searchParams;
  const usage = await getUsage();
  const days = range === "all" ? usage.perDay : usage.perDay.slice(-30);
  const chartData = days.map((day) => ({ day: day.day, fresh: day.fresh, cache: day.cacheRead + day.cacheWrite }));
  const topSessions = usage.sessions.slice(0, 8);
  const allTimeTotal = usage.allTime.total || 1;

  return (
    <>
      <div className="page-head">
        <div>
          <h1 className="page-title">Token usage</h1>
          <p className="page-subtitle">Read from {usage.dbPath}</p>
        </div>
        <RefreshButton />
      </div>

      <div className="grid-stats">
        <Stat label="Today, all tokens" value={usage.today.total} meta={`${formatCompact(usage.today.cacheRead)} cache read`} />
        <Stat label="Today, fresh" value={usage.today.fresh} meta={`${formatPercent(usage.today.fresh, usage.today.total || 1)} of today`} />
        <Stat label="All time, all tokens" value={usage.allTime.total} meta={`${formatNumber(usage.messageCount)} assistant messages`} />
        <Stat label="All time, fresh" value={usage.allTime.fresh} meta={`${formatPercent(usage.allTime.fresh, allTimeTotal)} of all time`} />
      </div>

      <section className="card" style={{ marginBottom: "var(--space-5)" }}>
        <div className="card-head">
          <h2 className="card-title">Daily usage</h2>
          <div className="nav" aria-label="Chart range">
            <Link href="/?range=30d" className="nav-link" data-active={range !== "all"}>Last 30 days</Link>
            <Link href="/?range=all" className="nav-link" data-active={range === "all"}>All time</Link>
          </div>
        </div>
        {chartData.length === 0 ? (
          <div className="empty">No assistant messages found in the database yet.</div>
        ) : (
          <UsageChart data={chartData} />
        )}
        <p className="stat-meta">Fresh means input, output and reasoning. Cache reads are shown separately.</p>
      </section>

      <section className="card">
        <div className="card-head">
          <h2 className="card-title">Sessions using the most tokens</h2>
          <Link href="/sessions" className="nav-link">All sessions</Link>
        </div>
        {topSessions.length === 0 ? (
          <div className="empty">No sessions yet.</div>
        ) : (
          <div className="table-wrap">
            <table className="table">
              <thead>
                <tr>
                  <th scope="col">Session</th>
                  <th scope="col">Model</th>
                  <th scope="col" className="num">Fresh</th>
                  <th scope="col" className="num">Cache read</th>
                  <th scope="col" className="num">Total</th>
                </tr>
              </thead>
              <tbody>
                {topSessions.map((session) => (
                  <tr key={session.key}>
                    <td className="ellipsis" title={session.label}>{session.label}</td>
                    <td className="ellipsis" title={session.model}>{session.model}</td>
                    <td className="num">{formatCompact(session.fresh)}</td>
                    <td className="num">{formatCompact(session.cacheRead)}</td>
                    <td className="num">{formatNumber(session.total)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </>
  );
}
