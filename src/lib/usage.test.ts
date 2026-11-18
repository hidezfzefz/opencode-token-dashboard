import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { openUsageDb } from "./db";
import { aggregate, readUsageRows, type UsageRow, type UsageSnapshot } from "./usage";

const NOW = new Date("2026-11-18T12:00:00Z");
const TODAY_MS = Date.UTC(2026, 10, 18, 10, 0, 0);
const YESTERDAY_MS = Date.UTC(2026, 10, 17, 10, 0, 0);

let dir: string;
let dbPath: string;
let rows: UsageRow[];
let snapshot: UsageSnapshot;

function message(id: string, time: number, input: number, output: number, cacheRead: number, model = "gpt-test") {
  return JSON.stringify({
    model: { id: model, providerID: "test", variant: "" },
    tokens: { input, output, reasoning: 0, cache: { read: cacheRead, write: 0 } },
  });
}

beforeAll(() => {
  dir = mkdtempSync(join(tmpdir(), "otd-"));
  dbPath = join(dir, "fixture.db");
  const db = new DatabaseSync(dbPath);
  db.exec(`
    CREATE TABLE session_message (id TEXT PRIMARY KEY, session_id TEXT, type TEXT, time_created INTEGER, data TEXT);
    CREATE TABLE session_v2 (id TEXT PRIMARY KEY, title TEXT, directory TEXT, project_id TEXT);
    CREATE TABLE project (id TEXT PRIMARY KEY, name TEXT, worktree TEXT);
  `);
  db.exec(`INSERT INTO project VALUES ('p1', 'alpha', '/work/alpha');`);
  db.exec(`INSERT INTO session_v2 VALUES ('s1', 'First session', '/work/alpha', 'p1');`);
  db.exec(`INSERT INTO session_v2 VALUES ('s2', 'Second session', '/work/alpha', 'p1');`);
  const insert = db.prepare("INSERT INTO session_message VALUES (?, ?, ?, ?, ?)");
  insert.run("m1", "s1", "assistant", TODAY_MS, message("m1", TODAY_MS, 10, 5, 100));
  insert.run("m2", "s1", "assistant", YESTERDAY_MS, message("m2", YESTERDAY_MS, 20, 6, 200, "other-model"));
  insert.run("m3", "s2", "user", TODAY_MS, "{}");
  db.close();
  rows = readUsageRows(dbPath);
  snapshot = aggregate(rows, dbPath, NOW);
});

afterAll(() => {
  rmSync(dir, { recursive: true, force: true });
});

describe("readUsageRows", () => {
  it("reads only assistant messages", () => {
    expect(rows).toHaveLength(2);
  });

  it("extracts token fields and joins session and project", () => {
    const row = rows.find((item) => item.modelId === "gpt-test")!;
    expect(row).toMatchObject({
      sessionTitle: "First session",
      projectLabel: "alpha",
      modelId: "gpt-test",
      input: 10,
      output: 5,
      cacheRead: 100,
    });
  });

  it("opens the database read-only", () => {
    const db = openUsageDb(dbPath);
    expect(() => db.exec("INSERT INTO project VALUES ('x', 'x', 'x')")).toThrow();
    db.close();
  });

  it("throws a clear error for a missing database", () => {
    expect(() => openUsageDb(join(dir, "missing.db"))).toThrow(/not found/);
  });
});

describe("aggregate", () => {
  it("computes all-time totals", () => {
    expect(snapshot.allTime).toMatchObject({ input: 30, output: 11, fresh: 41, cacheRead: 300, total: 341 });
  });

  it("computes today's totals separately", () => {
    expect(snapshot.today).toMatchObject({ input: 10, output: 5, fresh: 15, cacheRead: 100, total: 115 });
  });

  it("buckets by local calendar day", () => {
    expect(snapshot.perDay).toEqual([
      expect.objectContaining({ day: "2026-11-17", input: 20 }),
      expect.objectContaining({ day: "2026-11-18", input: 10 }),
    ]);
  });

  it("groups by session, model and project", () => {
    expect(snapshot.sessions).toHaveLength(1);
    expect(snapshot.sessions[0]).toMatchObject({ label: "First session", messages: 2, total: 341 });
    expect(snapshot.models.map((m) => m.key)).toEqual(["test/other-model", "test/gpt-test"]);
    expect(snapshot.projects).toHaveLength(1);
    expect(snapshot.projects[0]).toMatchObject({ label: "alpha", total: 341 });
  });

  it("handles empty input", () => {
    const empty = aggregate([], dbPath, NOW);
    expect(empty.allTime.total).toBe(0);
    expect(empty.perDay).toEqual([]);
  });
});
