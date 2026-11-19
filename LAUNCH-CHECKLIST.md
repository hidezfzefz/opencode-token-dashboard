# Launch checklist — v2

## Deploy
- [x] Release build passes with no unexplained warnings.
- [x] No secrets or credentials exist; `OPENCODE_DB_PATH`, `OTD_CONFIG_DIR` and `OTD_AUTO_REFRESH_SECONDS` are local settings, and `.env*` is gitignored.
- [n/a] Migrations and backups — the app never writes to a database; it opens OpenCode's DB read-only.
- [n/a] Domain, HTTPS, redirects and HSTS — local binary on the user's machine; no cloud deployment by design.
- [n/a] Preview/staging indexing — no hosted previews.

## API and sharing
- [x] Machine-readable contract published in `openapi.yaml`; curl examples and config format in `API.md`.
- [x] Versioned routes `/api/v1`; consistent camelCase JSON and one error format.
- [x] Health checks at `/health/live` and `/health/ready`; request IDs and structured logs.
- [x] Favicon and a useful 404 page; page titles, `lang="en"` and meta description set.
- [n/a] Open Graph, sitemap, structured data, robots.txt — local tool, not indexed.

## Commerce / email / legal
- [n/a] No store, payments, email sending, visitor data or cookies. No privacy policy or cookie consent needed for a local-only tool.

## Observability
- [x] Request logging with IDs; health endpoints for liveness/readiness.
- [n/a] External error tracking, uptime alerts and analytics — no hosted service to monitor; local logs are available.

## Handover
- [x] README with install/run options for Linux, Windows and macOS, API section, and rollback instructions.
- [x] SECURITY-REVIEW.md, TEST-PLAN.md and TEST-REPORT.md up to date.
- [x] CI matrix builds, lints, tests and packages binaries for all three platforms.
- [x] Pushed to GitHub.

## Open
- WebKit E2E (missing host libraries).
- Automated GUI screenshot tests in CI.
