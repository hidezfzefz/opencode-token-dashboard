# Security review

## Attack surface
- Native GUI: local file reads only; no network listener unless `--web` is used.
- Optional web mode: four GET pages, two CSV downloads, one search form.
- No auth, uploads, cookies, telemetry or secrets.

## Findings

| Finding | Severity | Status |
|---|---|---|
| Database writes | High | Fixed: `SQLITE_OPEN_READ_ONLY`; tested. |
| SQL injection | High | Fixed: static SQL; search filters in memory. |
| XSS in web mode | Medium | Fixed: Tera escapes by default; only generated SVG uses `|safe`. |
| Input handling | Medium | Sort key checked against a whitelist; search is a substring filter. |
| Security headers (web mode) | Low | Fixed: nosniff, DENY framing, no-referrer, permissions-policy. |
| Local database path displayed | Low | Accepted: local single-user tool. |
| WebKit untested | Low | Open: needs root-level host libraries. |

## Dependencies
- No runtime JavaScript dependencies; `legacy-nextjs/` is not built or shipped.
- Rust dependencies pinned by the committed `Cargo.lock`.
- SQLite compiled into the binary from source.

## Conclusion
No critical or high findings remain. The native app is safe for local use. Web mode should not be exposed to untrusted networks without authentication.
