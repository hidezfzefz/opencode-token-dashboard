# Test plan — v2 API, web UI and native GUI

## API endpoints (`cargo test` integration tests + live smoke)
| Endpoint | Happy path | Edge cases |
|---|---|---|
| `GET /api/v1/usage` | 200, camelCase snapshot, exact totals from fixture DB | missing DB → 500 with `error.code=database_error` |
| `GET /api/v1/usage/daily` | `range=all` and `30d`, newest first | default range is `30d`; unknown range falls back |
| `GET /api/v1/usage/sessions` | 200, all sessions, newest first | `q` filters; whitelisted `sort`/`dir`; invalid sort falls back to total |
| `GET /api/v1/usage/costs` | 200 with `configured:false` and empty rows when no pricing | priced models produce `totalUsd` |
| `GET /api/v1/settings` | 200 with persisted settings | missing file → defaults |
| `PUT /api/v1/settings` | 200, settings saved, idempotent | invalid interval → 422; unknown field → 422; malformed JSON → 422; body unchanged on failure |
| `GET /health/live` | 200 `{"status":"ok"}` | — |
| `GET /health/ready` | 200, database ok | missing DB → 503 |

Auth and rate limits are not applicable: the server binds to 127.0.0.1 and has no auth endpoints. This is recorded in SECURITY-REVIEW.md.

Load test: concurrent requests against the two busiest routes (`/api/v1/usage`, `/api/v1/usage/sessions`) with a scripted parallel curl loop; record status codes and timings.

## Web UI (Playwright, Chromium + Firefox; WebKit if host libraries exist)
- Four pages at 360, 390, 768, 1024, 1440 and 1920 px: HTTP 200, heading present, no horizontal document scroll.
- Search `?q=greeting` filters sessions; auto-refresh is absent during search.
- Both CSV downloads, link crawl, 404 page.
- Console errors and failed requests must be zero.
- Screenshots at 390 and 1440 px, inspected.

## Native GUI
- Builds and launches; process stays alive (no crash); Costs tab and non-blocking load verified by code inspection and launch check.

## Accessibility and design
- axe/Lighthouse accessibility checks where tooling is available.
- Hallmark sloplint on the web CSS/templates: 0 Floor failures.

## Rust checks
`cargo fmt -- --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, `cargo test`, `cargo build --release`.
