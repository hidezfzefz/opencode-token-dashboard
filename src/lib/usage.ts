import { io } from "next/cache";
import { defaultDbPath, openUsageDb } from "./db";

export interface Counts {
  input: number;
  output: number;
  reasoning: number;
  cacheRead: number;
  cacheWrite: number;
}

export interface Totals extends Counts {
  fresh: number;
  total: number;
}

export interface UsageRow extends Counts {
  sessionId: string;
  timeCreated: number;
  modelId: string;
  providerId: string;
  variant: string;
  sessionTitle: string;
  projectId: string;
  projectLabel: string;
}

export interface NamedTotals extends Totals {
  key: string;
  label: string;
  messages: number;
}

export interface SessionTotals extends NamedTotals {
  updatedAt: number;
  model: string;
  project: string;
}

export interface UsageSnapshot {
  dbPath: string;
  messageCount: number;
  today: Totals;
  allTime: Totals;
  perDay: Array<Totals & { day: string }>;
  sessions: SessionTotals[];
  models: NamedTotals[];
  projects: NamedTotals[];
}

const ZERO: Counts = {
  input: 0,
  output: 0,
  reasoning: 0,
  cacheRead: 0,
  cacheWrite: 0,
};

function withTotals(counts: Counts): Totals {
  return {
    ...counts,
    fresh: counts.input + counts.output + counts.reasoning,
    total: counts.input + counts.output + counts.reasoning + counts.cacheRead + counts.cacheWrite,
  };
}

function add(target: Counts, row: Counts): void {
  target.input += row.input;
  target.output += row.output;
  target.reasoning += row.reasoning;
  target.cacheRead += row.cacheRead;
  target.cacheWrite += row.cacheWrite;
}

export function dayKey(timeCreated: number): string {
  const date = new Date(timeCreated);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function readUsageRows(path: string): UsageRow[] {
  const db = openUsageDb(path);
  try {
    const result = db.prepare(`
      SELECT
        m.session_id AS sessionId,
        m.time_created AS timeCreated,
        COALESCE(json_extract(m.data, '$.model.id'), 'unknown') AS modelId,
        COALESCE(json_extract(m.data, '$.model.providerID'), 'unknown') AS providerId,
        COALESCE(json_extract(m.data, '$.model.variant'), '') AS variant,
        CAST(COALESCE(json_extract(m.data, '$.tokens.input'), 0) AS INTEGER) AS input,
        CAST(COALESCE(json_extract(m.data, '$.tokens.output'), 0) AS INTEGER) AS output,
        CAST(COALESCE(json_extract(m.data, '$.tokens.reasoning'), 0) AS INTEGER) AS reasoning,
        CAST(COALESCE(json_extract(m.data, '$.tokens.cache.read'), 0) AS INTEGER) AS cacheRead,
        CAST(COALESCE(json_extract(m.data, '$.tokens.cache.write'), 0) AS INTEGER) AS cacheWrite,
        COALESCE(NULLIF(s.title, ''), m.session_id) AS sessionTitle,
        COALESCE(p.id, 'none') AS projectId,
        COALESCE(NULLIF(p.name, ''), NULLIF(p.worktree, ''), s.directory, 'Other') AS projectLabel
      FROM session_message m
      LEFT JOIN session_v2 s ON s.id = m.session_id
      LEFT JOIN project p ON p.id = s.project_id
      WHERE m.type = 'assistant'
      ORDER BY m.time_created ASC
    `).all() as unknown as UsageRow[];
    return result;
  } finally {
    db.close();
  }
}

export function aggregate(rows: UsageRow[], path: string, now = new Date()): UsageSnapshot {
  const allTime: Counts = { ...ZERO };
  const today: Counts = { ...ZERO };
  const days = new Map<string, Counts>();
  const sessions = new Map<string, SessionTotals>();
  const models = new Map<string, NamedTotals>();
  const projects = new Map<string, NamedTotals>();
  const todayKey = dayKey(now.getTime());

  for (const row of rows) {
    const counts: Counts = {
      input: Number(row.input) || 0,
      output: Number(row.output) || 0,
      reasoning: Number(row.reasoning) || 0,
      cacheRead: Number(row.cacheRead) || 0,
      cacheWrite: Number(row.cacheWrite) || 0,
    };

    add(allTime, counts);

    const key = dayKey(Number(row.timeCreated));
    if (!days.has(key)) days.set(key, { ...ZERO });
    add(days.get(key)!, counts);
    if (key === todayKey) add(today, counts);

    const sessionKey = row.sessionId || "unknown";
    if (!sessions.has(sessionKey)) {
      sessions.set(sessionKey, {
        key: sessionKey,
        label: row.sessionTitle || sessionKey,
        ...ZERO,
        fresh: 0,
        total: 0,
        messages: 0,
        updatedAt: Number(row.timeCreated) || 0,
        model: `${row.providerId}/${row.modelId}${row.variant ? `#${row.variant}` : ""}`,
        project: row.projectLabel || "Other",
      });
    }
    const session = sessions.get(sessionKey)!;
    add(session, counts);
    session.messages += 1;
    session.updatedAt = Math.max(session.updatedAt, Number(row.timeCreated) || 0);

    const modelKey = `${row.providerId}/${row.modelId}${row.variant ? `#${row.variant}` : ""}`;
    if (!models.has(modelKey)) {
      models.set(modelKey, { key: modelKey, label: modelKey, ...ZERO, fresh: 0, total: 0, messages: 0 });
    }
    add(models.get(modelKey)!, counts);
    models.get(modelKey)!.messages += 1;

    const projectKey = row.projectId || "none";
    if (!projects.has(projectKey)) {
      projects.set(projectKey, { key: projectKey, label: row.projectLabel || "Other", ...ZERO, fresh: 0, total: 0, messages: 0 });
    }
    add(projects.get(projectKey)!, counts);
    projects.get(projectKey)!.messages += 1;
  }

  const finalize = <T extends Counts & { fresh: number; total: number }>(item: T): T => {
    const totals = withTotals(item);
    return { ...item, fresh: totals.fresh, total: totals.total };
  };

  const byTotal = <T extends { total: number }>(a: T, b: T) => b.total - a.total;

  return {
    dbPath: path,
    messageCount: rows.length,
    today: withTotals(today),
    allTime: withTotals(allTime),
    perDay: [...days.entries()]
      .map(([day, counts]) => finalize({ day, ...counts, fresh: 0, total: 0 }))
      .sort((a, b) => a.day.localeCompare(b.day)),
    sessions: [...sessions.values()].map(finalize).sort(byTotal),
    models: [...models.values()].map(finalize).sort(byTotal),
    projects: [...projects.values()].map(finalize).sort(byTotal),
  };
}

export async function getUsage(): Promise<UsageSnapshot> {
  await io();
  const path = defaultDbPath();
  return aggregate(readUsageRows(path), path);
}
