# Security review — opencode-token-dashboard

Date: 2026-11-18. Scope: the local Next.js app and its git repository. No production deployment exists.

## Attack surface
- Four read-only pages (`/`, `/sessions`, `/models`, `/projects`).
- One GET form (session search) and a refresh button that calls `router.refresh()`.
- No API routes, no file uploads, no webhooks, no auth endpoints, no admin area.
- No secrets or credentials in source, git history, or client bundles. The only configuration is `OPENCODE_DB_PATH`, an env var read server-side; it is never exposed as a `NEXT_PUBLIC_*` value.
- All data comes from the local OpenCode SQLite database. No prompt text or message content is read or rendered — only token counters, model ids, session titles and project labels.

## Findings

| Finding | Severity | Where | Fix | Status |
|---|---|---|---|---|
| 16 high npm audit findings from picomatch ReDoS (GHSA-fc5q-39xc-vjpf) | High (dev only) | eslint / vitest dev toolchain (`fast-glob`, `micromatch`, `tinyglobby`, `fdir`) | None available: advisory covers all versions `<= 4.0.7`, which is the latest. `npm audit fix` cannot resolve it. No fix is needed in production code; `npm audit --omit=dev` reports 0 vulnerabilities and none of these packages are bundled into the running app. | Open, accepted with reason |
| Semgrep could not run: engine killed (exit -11) even with `-j 1 --max-memory 800` | Low | Scanner environment | Manual review of all input handling and data access performed instead; see below. | Open, tool limitation |
| gitleaks and osv-scanner not installed in this environment | Low | Scanner environment | Manual `git grep` for secrets and `npm audit` cover the same ground. | Open, tool limitation |
| Local database path is displayed on the overview page | Low | `src/app/page.tsx` | Deliberate: this is a single-user local tool and the path helps confirm which DB is being read. Not exposed over a network in v1. | Accepted |
| Error boundary renders `error.message` | Low | `src/app/error.tsx` | Message may include a local file path. No stack trace is rendered. Local tool only. | Accepted |

## Checks that passed
- **Read-only database access:** `new DatabaseSync(path, { readOnly: true })`; a test proves an INSERT throws. (`src/lib/usage.test.ts`)
- **SQL injection:** all SQL is static; no user input is interpolated. The only query reads from fixed table names with fixed columns.
- **Input handling:** `q` is used only for a case-insensitive substring filter on in-memory strings. `sort` is checked against a fixed column whitelist; `dir` is restricted to `asc`/`desc`. No `dangerouslySetInnerHTML`, `innerHTML`, or user-supplied URLs.
- **No SSRF / no outbound requests:** the app makes no network calls. Recharts runs entirely in the browser with local data.
- **Secrets:** `git grep` for key/token/private-key patterns found nothing; no `.env` files are tracked and `.gitignore` excludes `.env*`.
- **Dependencies:** production dependencies have 0 known vulnerabilities; `package-lock.json` is committed.
- **Headers:** `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, `Permissions-Policy`, and `X-Powered-By` removed, set in `next.config.ts`.
- **Build-time validation:** `npm run lint`, `npx tsc --noEmit`, `npm test` (14 tests), and `npm run build` all pass.

## Conclusion
No critical or high findings in shipped application code. The only high-severity audit entries are in development tooling with no available patch and no exposure in the production bundle. The app is safe to run locally; it is not designed to be exposed to untrusted networks without adding authentication and network controls.
