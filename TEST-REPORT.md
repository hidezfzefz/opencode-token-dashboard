# Test report — v2 API, web UI and native app

Date: 2026-11-19.

## Results

| Check | Command / tool | Result |
|---|---|---|
| Format | `cargo fmt -- --check` | pass |
| Lint | `cargo clippy --all-targets --all-features --locked -- -D warnings` | pass, 0 warnings |
| Unit + integration tests | `cargo test` | 22 passed (7 endpoint integration tests) |
| Release build | `cargo build --release` | pass |
| Web E2E | `node legacy-nextjs/e2e-v2.mjs` | Chromium + Firefox: 48 page loads (4 pages × 6 widths), 0 failures, 0 console errors |
| API smoke | Playwright `page.request` | 7 endpoints all 200, all JSON |
| Lighthouse | in-app browser | accessibility **1.00**, best practices **1.00**, SEO **1.00**, 0 failures |
| Design lint | hallmark sloplint on `static/style.css` + rendered page | 0 FAIL, 0 WARN, 1 waived (declared dark surface) |
| Load test | 60 concurrent requests, 10 parallel, two busiest routes | 60/60 HTTP 200, average 2.45 s, max 5.4 s |

## Endpoint coverage
Every `/api/v1` endpoint was tested against a real fixture database in-process: success, invalid settings values, unknown fields, malformed bodies, missing database error format, sorting/filtering fallbacks, health checks, and costs with no pricing file. There is no auth or rate limiting by design (localhost only); this is documented in SECURITY-REVIEW.md.

## Bugs found and fixed during testing
1. Tests needed `Content-Type: application/json` for PUT; the test helper was fixed and now sends it.
2. The sessions fixture had only one session, so sort assertions were meaningless; a second session was added.
3. The first E2E run counted expected `ERR_ABORTED` CSV downloads as failures; the check now ignores aborted `.csv` requests.
4. Lighthouse scored accessibility 0.94 because of `<meta http-equiv="refresh">`. Replaced with a pause-aware JavaScript timer that stops while the tab is hidden or an input is focused; Lighthouse now scores 1.00 with no failures.
5. Load testing showed ~4.18 s average response time. Added a fingerprint-based snapshot cache in `AppState`; unchanged database state now reuses the last snapshot (average dropped to ~2.45 s, and repeat single reads are effectively instant).
6. Clippy flagged a complex cache type; extracted a `SnapshotCache` alias.

## Still open
- WebKit E2E: the browser binary exists but host libraries are missing; installing them requires root.
- Automated GUI screenshot testing in CI (needs a virtual display).
- No hosted preview was used: this app reads a local database and is not deployable to a public host.
