# Launch checklist — opencode-token-dashboard

## Deploy
- [x] Production build passes with no unexplained warnings — `npm run build`.
- [x] No secrets or environment credentials exist; `OPENCODE_DB_PATH` is an optional local path read only on the server. `.env*` is gitignored.
- [n/a] Database migrations and production backups — the app never writes to a database; it opens OpenCode's existing DB read-only.
- [n/a] Domain, HTTPS, www/apex redirect and HSTS — the app is local-first and intentionally not deployed to a cloud host in v1.
- [n/a] Preview/staging `noindex` — no preview deployments.

## Search and sharing
- [x] Page metadata and `lang="en"` set in the root layout. There is no canonical URL because the app has no public domain.
- [n/a] Open Graph / Twitter card image — no public sharing surface.
- [n/a] robots.txt, sitemap.xml, structured data — local-only tool, not indexed.
- [x] Favicon via `src/app/icon.svg` and a useful `not-found.tsx` page.

## Commerce / email
- [n/a] No store, payments, webhooks, tax or shipping.
- [n/a] No email sending.

## Legal and privacy
- [n/a] No visitor data is collected, no cookies, no analytics. The app runs on the user's own machine and renders only their local usage metadata.

## Observability
- [n/a] Error tracking and uptime checks — no public service to monitor. Errors are shown in the app's error boundary and logged to the browser console.
- [n/a] Analytics — none by design.

## Handover
- [x] README with local run instructions, configuration, tests, rollback and limitations.
- [x] TEST-PLAN.md, TEST-REPORT.md and SECURITY-REVIEW.md written and committed.
- [x] Source pushed to GitHub (see the repository URL in the final handover).

## Still open (not blocking a local tool)
- WebKit cross-browser run — browser binary installed, but host libraries are missing and installing them requires root.
- Unpatchable dev-only `picomatch` advisory (documented in SECURITY-REVIEW.md).
