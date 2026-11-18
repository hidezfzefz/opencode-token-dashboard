# opencode-token-dashboard: plan

## Goal
Turn the Rust dashboard into a fully native desktop app for Linux, Windows and macOS, while keeping the browser version available.

## Architecture
- `src/gui.rs` — egui app: tabs, tables, search, sort, custom-painted chart, CSV export.
- `src/main.rs` — default native GUI; `--web` starts the Axum server.
- `src/usage.rs` — shared read-only data layer with per-OS default database paths.
- `src/chart.rs`, `src/format.rs`, `src/csv.rs` — shared pure helpers.
- `templates/`, `static/` — preserved website for web mode.

## Done
1. Added eframe/egui and implemented the native window. ✓
2. Reused the data layer, aggregation, formatting and CSV helpers. ✓
3. Implemented all four tabs, refresh, search, sort, chart and CSV export. ✓
4. Made the default database path platform-aware (Windows/macOS/Linux). ✓
5. Verified the window launches and stays open on this machine. ✓
6. CI matrix builds and tests on Ubuntu, Windows and macOS and uploads binaries. ✓
7. Auto sync: GUI watches database/WAL fingerprints and reloads on change; web mode meta refresh with configurable interval. ✓

## Open
- No GUI screenshot test in CI (would need a virtual display).
