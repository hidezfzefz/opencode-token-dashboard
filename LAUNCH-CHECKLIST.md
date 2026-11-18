# Launch checklist

## Build
- [x] Native GUI builds on Linux and is verified to launch.
- [x] CI matrix builds and tests on Ubuntu, Windows and macOS; binaries uploaded as artifacts.
- [x] fmt, clippy and 10 tests pass.
- [x] No Node runtime, webview, external database or secrets required.

## Functionality
- [x] Four tabs, refresh, search, sort, chart and CSV export in the native app.
- [x] Original website preserved in `--web` mode.
- [x] Platform-specific default database paths.

## Security
- [x] Read-only database access, tested.
- [x] Security headers in web mode.
- [x] No network calls or data collection.

## Handover
- [x] README with install/run instructions for all three platforms.
- [x] TEST-PLAN.md, TEST-REPORT.md and SECURITY-REVIEW.md updated.
- [x] Pushed to GitHub.

## Open
- Automated GUI screenshot tests in CI.
- WebKit browser test for web mode.
