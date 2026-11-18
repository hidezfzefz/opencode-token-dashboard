import type { SessionTotals, Totals } from "./usage";

const SESSION_HEADER = [
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

const DAILY_HEADER = ["Day", "Fresh", "Input", "Output", "Reasoning", "Cache read", "Cache write", "Total"];

export function escapeCsvCell(value: string | number): string {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
}

export function toCsv(header: string[], rows: Array<Array<string | number>>): string {
  return [header, ...rows].map((row) => row.map(escapeCsvCell).join(",")).join("\n");
}

export function sessionsToCsv(rows: SessionTotals[]): string {
  return toCsv(
    SESSION_HEADER,
    rows.map((row) => [
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
    ]),
  );
}

export function dailyToCsv(rows: Array<Totals & { day: string }>): string {
  return toCsv(
    DAILY_HEADER,
    rows.map((row) => [
      row.day,
      row.fresh,
      row.input,
      row.output,
      row.reasoning,
      row.cacheRead,
      row.cacheWrite,
      row.total,
    ]),
  );
}
