# opencode-token-dashboard: brief (v2 rework)

**What it is:** A native Rust desktop app for OpenCode token usage on Linux, Windows and macOS, with a local JSON API and an optional browser dashboard.
**For:** The developer running OpenCode locally. First action: open the native window and see current usage; data stays up to date automatically.
**Success:** One toolchain and one data layer for GUI, web UI and API; all existing features preserved; cross-platform CI green; committed to the existing GitHub repo.

## Scope
- Native GUI (egui): overview, sessions, models, projects, daily chart, CSV export, auto sync with configurable interval, persistent settings.
- Web mode (`--web`): original Tera website, auto-refresh, CSV downloads.
- JSON API (local, no auth):
  - `GET /api/usage` — full snapshot (totals, per-day, sessions, models, projects, comparison stats)
  - `GET /api/usage/daily` — per-day rows for the selected range
  - `GET /api/usage/sessions` — searchable, sortable session rows
  - `GET /api/usage/costs` — cost estimates from a user-supplied pricing file
  - `GET /api/settings`, `PUT /api/settings` — read/write auto-sync interval and chart range
- Cost estimates use **only** user-supplied rates in `pricing.json`; no prices are invented or fetched.

## Design
- Preserve the dark, calm Convio Design look in both GUIs.
- Shared library crate (`src/lib.rs`) with pure modules; two thin frontends (GUI, web/API) in one binary.
- GUI loads data without blocking the window; auto sync watches DB/WAL file fingerprints.

## Stack
- Rust, Axum/Tokio, eframe/egui, rusqlite (bundled SQLite), Tera, Chrono, Serde.
- Local SQLite only; no hosted database, no cloud deployment, no Node runtime.

## Constraints and assumptions
- Consumers are local only (native app + browser dashboard); no authentication required on localhost.
- OpenCode database path is configurable via `OPENCODE_DB_PATH` / `--db-path`, with per-OS defaults.
- Settings and pricing live in a per-OS config directory; no secrets are stored.
- Convio's pipeline defaults (Hono, Cloudflare Workers, hosted Postgres) are set aside because they do not fit a local native app; the project's own stack wins.

## Out of scope
- Auth, telemetry, remote hosting, writing to the OpenCode database, displaying prompt content.
