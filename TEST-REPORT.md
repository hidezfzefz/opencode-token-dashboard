# Test report

| Check | Result |
|---|---|
| `cargo fmt -- --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass, 0 warnings |
| `cargo test` | 10 passed, 0 failed |
| `cargo build --release` (Linux) | pass |
| Native window launch on Linux | process stayed alive for the full timeout (no crash) |
| `--web` mode after GUI changes | HTTP 200 on `/` |
| Browser E2E (earlier, Chromium + Firefox) | 48 page loads, 0 failures, 0 console errors |

## Fixes during the GUI work
1. egui 0.37 API changes: `App::ui` replaced `App::update`, themed style via `set_style_of`, `Rect::from_min_max`, progress-bar API.
2. Clippy: boxed the large snapshot enum variant and used struct-update style initialization.
3. Platform-aware default database paths for Windows and macOS.
4. CI now builds and tests on Ubuntu, Windows and macOS and uploads each binary.

## Open
- Automated GUI screenshot testing (needs a virtual display in CI).
- WebKit browser testing (needs root-level host libraries).
