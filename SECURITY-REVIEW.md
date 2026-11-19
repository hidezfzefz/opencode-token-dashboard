# Security review — v2 API and native app

Date: 2026-11-19. Scope: the native GUI, the `--web` server and the `/api/v1` JSON API. No public deployment exists.

## Attack surface
- Native GUI: local file reads only; no listener.
- Web mode: HTML pages, CSV downloads, six `/api/v1` endpoints and two health endpoints.
- One state-changing endpoint: `PUT /api/v1/settings`, which writes only two local UI fields.
- Inputs: query parameters (`q`, `sort`, `dir`, `range`), a JSON settings body, and environment variables/paths. No uploads, webhooks or third-party scripts.
- No secrets, tokens or credentials exist in the app.

## Findings

| Finding | Severity | Where | Status |
|---|---|---|---|
| No authentication/authorisation | High if exposed | all `/api/v1` routes | Accepted with guard: server binds to `127.0.0.1` by default and is intended for one local user. A warning is now printed if `--host` is set to a non-loopback address. It must not be exposed to an untrusted network. |
| No rate limiting | Low | API | Accepted: no auth endpoints, no remote clients, no expensive write operations. Read requests do scan a local DB, but only localhost can reach them. |
| CORS | Low | API | No CORS headers are emitted, so browsers apply same-origin policy by default. No origin is reflected and credentials are not used. |
| Mass assignment | Medium | `PUT /api/v1/settings` | Fixed: `Settings` uses `deny_unknown_fields`, only `autoSyncSeconds` and `allDays` exist, and `is_valid()` restricts the interval to an allowlist. Invalid bodies return 422 and change nothing. |
| SQL injection | High if present | data layer | Fixed: all SQL is static; no user text is interpolated. Search filters in memory. |
| Database writes | High if present | SQLite connection | Fixed: `SQLITE_OPEN_READ_ONLY`; a test proves INSERT fails. |
| XSS | Medium | web templates | Tera escapes by default; the only `|safe` value is an SVG generated from numeric data and dates from the DB. API responses are JSON. |
| SSRF | Medium if present | none | No outbound network requests exist. |
| Local path disclosure | Low | `/api/v1/usage` `dbPath`, overview page | Accepted: local single-user tool; helps identify which DB is in use. |
| Error message disclosure | Low | 500 JSON errors | Only a database error string is returned; no stack traces, no secrets. |
| WebKit untested | Low | test coverage | Open: host libraries missing; installing them needs root. |

## Scanners and checks
- `semgrep scan --config p/default -j 1 --max-memory 600 src` — clean, exit 0.
- `git grep` over tracked files for keys, tokens and private-key markers — nothing found.
- Tracked `.env` files — none; `.env*` is gitignored.
- `Cargo.lock` committed; no runtime npm dependencies are shipped (the archived `legacy-nextjs/` tree is not built by CI).
- gitleaks, osv-scanner and cargo-audit are not installed in this environment; manual history/code checks above cover their main use here.
- `cargo clippy --all-targets --all-features --locked -- -D warnings` — clean.
- Integration tests cover strict JSON errors, settings validation, the read-only database and health endpoints.

## Headers
Web responses set `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` and `Permissions-Policy`. API responses also carry `x-request-id`; request logs contain method, path, status and duration only — no bodies or tokens.

## Conclusion
No critical or high finding remains in shipped code. The only high-risk condition is deliberately exposing the unauthenticated local API to an untrusted network; the default loopback binding and startup warning mitigate it. The app is safe for its intended local use.
