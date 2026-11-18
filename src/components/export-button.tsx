"use client";

import { Download } from "lucide-react";

interface ExportButtonProps {
  csv: string;
  filename: string;
  label?: string;
  disabled?: boolean;
}

export function ExportButton({ csv, filename, label = "Export CSV", disabled = false }: ExportButtonProps) {
  function download() {
    const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    link.remove();
    URL.revokeObjectURL(url);
  }

  return (
    <button type="button" className="btn" onClick={download} disabled={disabled}>
      <Download size={15} strokeWidth={1.5} aria-hidden="true" />
      {label}
    </button>
  );
}
