# Launch checklist — Rust implementation

## Build
- [x] Release build passes with no warnings.
- [x] rustfmt and clippy clean; 10 tests pass.
- [x] Binary is self-contained: bundled SQLite, no Node runtime, no database server.
- [x] No secrets or credentials; database path is an optional local argument/env var.
- [x] No migrations or backups needed: the app never writes to a database.

## Website
- [x] Same four pages, CSV downloads, 404 and error page as the original.
- [x] Same design tokens and CSS, embedded in the binary.
- [x] Favicon embedded; page titles and `lang` set.
- [x] Responsive from 360 to 1920 px in Chromium and Firefox.

## Security
- [x] Read-only database access, tested.
- [x] Security headers on every response.
- [x] No network calls, telemetry, cookies or user data collection.

## Handover
- [x] README with Gentoo build/run instructions and an OpenRC service file.
- [x] TEST-PLAN.md, TEST-REPORT.md and SECURITY-REVIEW.md updated for Rust.
- [x] CI workflow builds and tests the Rust app.
- [x] Pushed to GitHub.

## Open
- WebKit cross-browser run (needs root-level host libraries).
- Cost-per-model estimates (needs a user-supplied pricing file).
