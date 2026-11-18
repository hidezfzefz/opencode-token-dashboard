# Test plan

## Rust (`cargo test`)
- Number formatting, CSV escaping, SVG chart helper, read-only database access, aggregation with a synthetic fixture database, and database fingerprint changes.

## Native GUI
- `cargo build --release` on Linux, Windows and macOS via CI matrix.
- Launch check on Linux: process stays alive with a real window (timeout exit 124, not a crash).
- Manual checks: four tabs, refresh, search, sort, chart, CSV export.

## Web mode (`--web`)
- Optional Playwright E2E in `legacy-nextjs/e2e-rust.mjs`: four routes at six widths in Chromium and Firefox, search, CSV downloads, no console errors.

## Checks
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release`
