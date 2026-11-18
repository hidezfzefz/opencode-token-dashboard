# Test report — Rust implementation

## Results

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt -- --check` | pass |
| Lint | `cargo clippy --all-targets -- -D warnings` | pass, 0 warnings |
| Unit + integration tests | `cargo test` | 10 passed, 0 failed |
| Release build | `cargo build --release` | pass; binary ~3.9 MB |
| E2E, Chromium and Firefox | `node legacy-nextjs/e2e-rust.mjs` | 48 page loads, 0 failures, 0 console errors |
| Data accuracy | displayed figures vs direct `sqlite3` aggregate | exact match |
| Security headers | `curl -D -` | nosniff, DENY framing, no-referrer, permissions-policy present |
| Error state | run with a missing `--db-path` | error page rendered |

## E2E details
- Four routes at six widths (360–1920 px) in both browsers, no horizontal page scroll.
- Search `?q=greeting` returned matching sessions.
- Session and daily CSV downloads produced the correct filenames.
- Screenshots at 390 px and 1440 px were inspected. A first pass showed distorted chart labels on mobile; fixed by generating a dedicated narrow SVG chart.

## Bugs found and fixed
1. Parallel Rust tests collided on a fixed fixture filename; paths now include nanoseconds.
2. `cargo fmt` and clippy warnings were resolved (sorting with `Reverse`, `&Path` instead of `&PathBuf`).
3. Mobile SVG chart text was horizontally squashed by viewBox scaling; a separate 390px chart is now rendered under 640px.
4. The last x-axis label was clipped; first and last labels now use start/end anchoring.
5. CI initially still used the Node workflow; replaced with a Rust toolchain workflow.
6. Stable clippy in CI flagged a manual modulo check that nightly allowed; replaced with `is_multiple_of(3)`. CI now passes all steps.

## Still open
- WebKit browser test: host libraries missing on this machine and installing them needs root.
- Cost-per-model estimates (would require a user-supplied pricing file).
