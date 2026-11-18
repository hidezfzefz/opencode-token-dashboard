import type { SessionTotals } from "./usage";

const HEADER = [
  "Session",
  "Last active",
  "Model",
  "Project",
  "Messages",
  "Input",
  "Output",
  "Reasoning",
  "Cache read",
  "Cache write",
  "Fresh",
  "Total",
];

export function escapeCsvCell(value: string | number): string {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
}

export function sessionsToCsv(rows: SessionTotals[]): string {
  const lines = rows.map((row) =>
    [
      row.label,
      new Date(row.updatedAt).toISOString(),
      row.model,
      row.project,
      row.messages,
      row.input,
      row.output,
      row.reasoning,
      row.cacheRead,
      row.cacheWrite,
      row.fresh,
      row.total,
    ]
      .map(escapeCsvCell)
      .join(","),
  );
  return [HEADER.join(","), ...lines].join("\n");
}
