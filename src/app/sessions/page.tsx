import { ExportButton } from "@/components/export-button";
import { RefreshButton } from "@/components/refresh-button";
import { formatCompact, formatNumber } from "@/lib/format";
import { getUsage, type SessionTotals } from "@/lib/usage";

export const instant = false;

type SortKey = "label" | "updatedAt" | "input" | "output" | "reasoning" | "cacheRead" | "fresh" | "total";

const columns: Array<{ key: SortKey; label: string; numeric: boolean }> = [
  { key: "label", label: "Session", numeric: false },
  { key: "updatedAt", label: "Last active", numeric: true },
  { key: "input", label: "Input", numeric: true },
  { key: "output", label: "Output", numeric: true },
  { key: "reasoning", label: "Reasoning", numeric: true },
  { key: "cacheRead", label: "Cache read", numeric: true },
  { key: "fresh", label: "Fresh", numeric: true },
  { key: "total", label: "Total", numeric: true },
];

function sortSessions(rows: SessionTotals[], key: SortKey, direction: "asc" | "desc"): SessionTotals[] {
  const factor = direction === "asc" ? 1 : -1;
  return [...rows].sort((a, b) => {
    const left = a[key];
    const right = b[key];
    if (typeof left === "string" || typeof right === "string") {
      return String(left).localeCompare(String(right)) * factor;
    }
    return (Number(left) - Number(right)) * factor;
  });
}

function hrefFor(key: SortKey, current: SortKey, direction: string, query: string): string {
  const nextDirection = current === key && direction === "desc" ? "asc" : "desc";
  const params = new URLSearchParams();
  if (query) params.set("q", query);
  params.set("sort", key);
  params.set("dir", nextDirection);
  return `/sessions?${params.toString()}`;
}

export default async function SessionsPage({
  searchParams,
}: {
  searchParams: Promise<{ q?: string; sort?: string; dir?: string }>;
}) {
  const { q = "", sort = "total", dir = "desc" } = await searchParams;
  const usage = await getUsage();
  const query = q.trim().toLowerCase();
  const filtered = query
    ? usage.sessions.filter((session) =>
        `${session.label} ${session.model} ${session.project}`.toLowerCase().includes(query),
      )
    : usage.sessions;
  const key = (columns.some((column) => column.key === sort) ? sort : "total") as SortKey;
  const direction = dir === "asc" ? "asc" : "desc";
  const rows = sortSessions(filtered, key, direction);

  return (
    <>
      <div className="page-head">
        <div>
          <h1 className="page-title">Sessions</h1>
          <p className="page-subtitle">{formatNumber(usage.sessions.length)} sessions with assistant messages</p>
        </div>
        <div className="nav" aria-label="Session actions">
          <ExportButton rows={rows} />
          <RefreshButton />
        </div>
      </div>

      <form action="/sessions" method="get" style={{ marginBottom: "var(--space-4)" }}>
        <input
          className="input"
          type="search"
          name="q"
          defaultValue={q}
          placeholder="Search sessions, models or projects"
          aria-label="Search sessions"
          style={{ maxWidth: 420 }}
        />
      </form>

      <section className="card">
        {rows.length === 0 ? (
          <div className="empty">
            {usage.sessions.length === 0
              ? "No sessions found in the database yet."
              : "No sessions match this search. Try a different term."}
          </div>
        ) : (
          <div className="table-wrap">
            <table className="table">
              <thead>
                <tr>
                  {columns.map((column) => (
                    <th key={column.key} scope="col" className={column.numeric ? "num" : undefined}>
                      <a href={hrefFor(column.key, key, direction, q)} className="nav-link" style={{ padding: 0 }}>
                        {column.label}
                      </a>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.map((session) => (
                  <tr key={session.key}>
                    <td className="ellipsis" title={session.label}>{session.label}</td>
                    <td className="num">{new Date(session.updatedAt).toLocaleDateString()}</td>
                    <td className="num">{formatCompact(session.input)}</td>
                    <td className="num">{formatCompact(session.output)}</td>
                    <td className="num">{formatCompact(session.reasoning)}</td>
                    <td className="num">{formatCompact(session.cacheRead)}</td>
                    <td className="num">{formatCompact(session.fresh)}</td>
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
