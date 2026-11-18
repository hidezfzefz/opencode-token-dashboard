# opencode-token-dashboard: brief

**What it is:** A fully native Rust desktop app for OpenCode token usage, running on Linux, Windows and macOS.
**For:** A developer who wants to see token usage without a browser, webview or Node runtime.
**Success:** One Rust binary per platform, opening a real native window with the same data and features as the earlier website. Web mode remains available via `--web`.

## Scope
- Native GUI: overview, sessions, models, projects, refresh, search, sort, CSV export.
- Read-only SQLite access to the OpenCode database, with per-OS default paths.
- Optional web server with the original website preserved.

## Stack
- Rust, eframe/egui (native GUI), Axum/Tokio (optional web mode), rusqlite with bundled SQLite, Tera, Chrono, Serde.

## Constraints
- No Node at runtime, no webview, no external database, no network calls.
- Same dark, calm visual language as the original website.
