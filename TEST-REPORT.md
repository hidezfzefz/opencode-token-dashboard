# Test report

| Check | Result |
|---|---|
| `cargo fmt -- --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass, 0 warnings |
| `cargo test` | 14 passed, 0 failed |
| `cargo build --release` (Linux) | pass |
| Native window launch on Linux | process stayed alive for the full timeout (no crash) |
| `--web` mode after GUI changes | HTTP 200 on `/` |
| Browser E2E (earlier, Chromium + Firefox) | 48 page loads, 0 failures, 0 console errors |

## Fixes during the GUI work
1. egui 0.37 API changes: `App::ui` replaced `App::update`, themed style via `set_style_of`, `Rect::from_min_max`, progress-bar API.
2. Clippy: boxed the large snapshot enum variant and used struct-update style initialization.
3. Platform-aware default database paths for Windows and macOS.
4. CI now builds and tests on Ubuntu, Windows and macOS and uploads each binary.

## Auto sync verification
- Database fingerprint changes when the underlying file changes: unit test passes.
- Web mode emits a pause-aware refresh script by default and no script when `OTD_AUTO_REFRESH_SECONDS=0`.
- Native window still launches with auto sync enabled; no crash.
- Settings save/load round-trip tested; invalid intervals normalized to 1 second.
- Web auto-refresh is absent on `/sessions?q=...` so typing in search is not interrupted.

## Latest improvement round
- Sessions can be filtered by exact model and project in both interfaces, in addition to free-text search. CSV exports and sort links preserve all filters. Added a pure `filter_sessions` helper with a unit test; 14 tests now pass.
- GUI keyboard shortcuts: keys 1–4 switch tabs, R refreshes.
- Replaced the web meta refresh with a pause-aware timer; axe-core reports zero violations on all four pages.
- Browser E2E re-run on Chromium and Firefox with 0 failures.

## Open
- Automated GUI screenshot testing (needs a virtual display in CI).
- WebKit browser testing (needs root-level host libraries).
