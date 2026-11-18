import { RefreshButton } from "@/components/refresh-button";
import { ShareBar } from "@/components/share-bar";
import { formatCompact, formatNumber, formatPercent } from "@/lib/format";
import { getUsage } from "@/lib/usage";

export const instant = false;

export default async function ModelsPage() {
  const usage = await getUsage();
  const rows = usage.models;
  const max = rows[0]?.total ?? 0;

  return (
    <>
      <div className="page-head">
        <div>
          <h1 className="page-title">Models</h1>
          <p className="page-subtitle">{rows.length} models and variants across all sessions</p>
        </div>
        <RefreshButton />
      </div>

      <section className="card">
        {rows.length === 0 ? (
          <div className="empty">No model data found in the database yet.</div>
        ) : (
          <div className="table-wrap">
            <table className="table">
              <thead>
                <tr>
                  <th scope="col">Model</th>
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
