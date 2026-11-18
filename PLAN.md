# opencode-token-dashboard: plan

## Goal
Keep the website; rebuild the implementation in Rust so it runs on Gentoo with no Node runtime.

## Architecture
- `src/main.rs` — CLI, template registration, server startup.
- `src/usage.rs` — read-only SQLite access and aggregation.
- `src/chart.rs` — server-side SVG chart (wide and narrow variants).
- `src/csv.rs`, `src/format.rs` — pure helpers.
- `src/web.rs` — Axum routes, query handling, Tera contexts, security headers.
- `templates/`, `static/` — the preserved website, embedded into the binary at compile time.

## Build order (done)
1. Archive the Next.js app into `legacy-nextjs/`; copy its CSS and icon into `static/`. ✓
2. Create the Cargo project with Axum, Tokio, rusqlite (bundled), Tera, Chrono and Serde. ✓
3. Port the data layer and aggregation, with fixture-DB tests. ✓
4. Recreate the four pages, CSV downloads, 404 and error pages in Tera. ✓
5. Port the chart to server-generated SVG, with a dedicated narrow variant. ✓
6. Add security headers, CLI options and Gentoo OpenRC service. ✓
7. Verify with cargo fmt, clippy, tests, release build and browser E2E on Chromium and Firefox. ✓
8. Replace CI with a Rust workflow; update all documentation. ✓

## Open
- WebKit browser testing still needs root-level host libraries.
