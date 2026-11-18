import { existsSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { DatabaseSync } from "node:sqlite";

export function defaultDbPath(): string {
  return process.env.OPENCODE_DB_PATH || join(homedir(), ".local", "share", "opencode", "opencode.db");
}

export function openUsageDb(path: string = defaultDbPath()): DatabaseSync {
  if (!existsSync(path) || !statSync(path).isFile()) {
    throw new Error(`OpenCode database not found at ${path}`);
  }
  return new DatabaseSync(path, { readOnly: true });
}
