import { RefreshButton } from "@/components/refresh-button";
import { ShareBar } from "@/components/share-bar";
import { formatCompact, formatNumber, formatPercent } from "@/lib/format";
import { getUsage } from "@/lib/usage";

export default async function ProjectsPage() {
  const usage = await getUsage();
  const rows = usage.projects;
  const max = rows[0]?.total ?? 0;

  return (
    <>
      <div className="page-head">
        <div>
          <h1 className="page-title">Projects</h1>
          <p className="page-subtitle">{rows.length} projects with assistant messages</p>
        </div>
        <RefreshButton />
      </div>

      <section className="card">
        {rows.length === 0 ? (
          <div className="empty">No project data found in the database yet.</div>
        ) : (
          <div className="table-wrap">
            <table className="table">
              <thead>
                <tr>
                  <th scope="col">Project</th>
                  <th scope="col" className="num">Messages</th>
                  <th scope="col" className="num">Fresh</th>
                  <th scope="col" className="num">Cache read</th>
                  <th scope="col" className="num">Total</th>
                  <th scope="col" className="num">Share</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => (
                  <tr key={row.key}>
                    <td className="ellipsis" title={row.label}>{row.label}</td>
                    <td className="num">{formatNumber(row.messages)}</td>
                    <td className="num">{formatCompact(row.fresh)}</td>
                    <td className="num">{formatCompact(row.cacheRead)}</td>
                    <td className="num">{formatNumber(row.total)}</td>
                    <td className="num">
                      <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: 8 }}>
                        <ShareBar value={row.total} max={max} label={row.label} />
                        <span>{formatPercent(row.total, usage.allTime.total || 1)}</span>
                      </div>
                    </td>
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
