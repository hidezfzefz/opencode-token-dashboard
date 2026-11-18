# Security review — Rust implementation

## Attack surface
- Four server-rendered pages, two CSV downloads, one search form, one 404 page.
- No API surface beyond those GET routes; no auth, no uploads, no webhooks, no cookies, no telemetry.
- Only configuration is `OPENCODE_DB_PATH` / `--db-path`, read server-side. No secrets exist.

## Findings

| Finding | Severity | Status |
|---|---|---|
| Database writes | High if present | Fixed: connection uses `SQLITE_OPEN_READ_ONLY`; a test proves INSERT fails. |
| SQL injection | High if present | Fixed: all SQL is static, no user input is interpolated. |
| XSS | Medium if present | Fixed: Tera escapes HTML by default; the only `|safe` value is an SVG generated from numeric data and day strings from the database. No user input reaches it. |
| Input handling | Medium | Query values are used only for case-insensitive in-memory filtering and a whitelisted sort key/direction. |
| Missing security headers | Low | Fixed: nosniff, DENY framing, no-referrer and permissions-policy on every response. |
| Local path shown on the overview page | Low | Accepted: a single-user local tool; the path helps confirm which database is read. |
| Error message shown on the error page | Low | Accepted: no stack trace, local tool only. |
| WebKit not tested | Low | Open: host libraries missing; needs root to install. |

## Dependency review
- No npm dependencies are used at runtime. The archived `legacy-nextjs/` tree is not built by CI or shipped in the binary.
- Rust dependencies are resolved from crates.io and pinned by the committed `Cargo.lock`.
- SQLite is compiled into the binary from source; there is no external database service.

## Conclusion
No critical or high findings remain in the shipped application. The app is safe to run locally. It is not designed for exposure to untrusted networks without adding authentication and network controls.
