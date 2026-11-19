# v2 rework plan — API contract and build order

## Contract
`openapi.yaml` is the source of truth. Versioning is in the URL (`/api/v1`). All error responses use `{"error":{"code","message"}}`. No auth or rate limits (localhost only). `PUT /api/v1/settings` is idempotent full replacement with strict field validation.

## Build order (each endpoint tested before the next)
1. `GET /api/v1/usage` — serialize the shared `UsageSnapshot` as JSON. Test: response matches the aggregate returned by the data layer.
2. `GET /api/v1/usage/daily` — range `30d|all`, newest first. Test: length and ordering match the snapshot's `per_day` slice.
3. `GET /api/v1/usage/sessions` — `q`, whitelisted `sort`, `dir`. Test: search filters, sort order, invalid sort falls back to total.
4. `GET /api/v1/usage/costs` — user-supplied pricing only. Test: empty rows with no pricing file; rates applied per million tokens.
5. `GET /api/v1/settings` — returns persisted settings. Test: round-trip with a temp config path.
6. `PUT /api/v1/settings` — strict JSON body, 422 on invalid fields/values. Test: valid update persists; invalid returns 422 and does not change the file.
7. GUI and web UI consume the same library functions (no duplicated aggregation).
8. Non-blocking GUI startup: load data on a worker thread, show the window immediately.
9. Security review, thorough tests, launch checklist, commit and push. ✓
10. Post-pipeline: usage insights in both GUIs and headless `report` / `export` commands for scripts and cron. ✓

## Cross-cutting rules
- One data layer in `src/lib.rs`; GUI and web/API are thin frontends.
- No `unwrap`/`expect` in request paths; return typed errors.
- Clippy with `--all-targets --all-features --locked -- -D warnings`; rustfmt; tests on all three OSes in CI.
- Keep the existing website, CSV downloads, auto sync and settings behaviour working throughout.
