# Test plan — Rust implementation

## Unit and integration tests (`cargo test`)
- Number formatting: grouped thousands, compact notation, percentages.
- CSV: escaping of commas, quotes and newlines; header and row layout.
- Chart: SVG contains both series and day labels; empty input returns nothing.
- Data layer: only assistant rows are read; an INSERT on the read-only connection fails; totals, day buckets, sessions, models, projects, yesterday and 7-day average are correct against a synthetic fixture database.

## Browser E2E (optional, `legacy-nextjs/e2e-rust.mjs`)
- All four pages at 360, 390, 768, 1024, 1440 and 1920 px: HTTP 200, heading present, no horizontal document scroll.
- Search `?q=greeting` filters sessions.
- Both CSV downloads start with the expected filename.
- No console errors or uncaught exceptions.
- Run on Chromium and Firefox.

## Checks
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release`
- Compare displayed figures against a direct `sqlite3` aggregate on the real database.
- Verify security headers with `curl -D -`.
