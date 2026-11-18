"use client";

import { Download } from "lucide-react";
import { sessionsToCsv } from "@/lib/csv";
import type { SessionTotals } from "@/lib/usage";

export function ExportButton({ rows }: { rows: SessionTotals[] }) {
  function download() {
    const csv = sessionsToCsv(rows);
    const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "opencode-token-usage-sessions.csv";
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  }

  return (
    <button type="button" className="btn" onClick={download} disabled={rows.length === 0}>
      <Download size={15} strokeWidth={1.5} aria-hidden="true" />
      Export CSV
    </button>
  );
}
