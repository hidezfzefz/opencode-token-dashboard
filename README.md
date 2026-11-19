# opencode-token-dashboard

A **fully native Rust desktop app** for OpenCode token usage. It opens as a real window on Linux, Windows and macOS — no browser, webview, Node.js or web server needed. It reads your OpenCode SQLite database read-only and shows today's and all-time totals, a daily chart, and breakdowns by session, model and project.

A browser version with a local JSON API is available with the `--web` flag, but the default is the native GUI. Both use the same Rust data layer.

The old Next.js implementation is archived in `legacy-nextjs/` for reference; it is not needed to build or run the app.

## Requirements

- Rust (stable; recent version needed for the crates used here)
- A C compiler (`cc` on Linux/macOS, MSVC on Windows) — `rusqlite` compiles SQLite from source
- Nothing else. No Node, no npm, no database server, no webview runtime.

## Install and run

The simplest cross-platform install:

```sh
git clone https://github.com/hidezfzefz/opencode-token-dashboard.git
cd opencode-token-dashboard
cargo install --path .
```

That puts `opencode-token-dashboard` in `~/.cargo/bin` (already on your PATH by default). Just run:

```sh
opencode-token-dashboard
```

A native window opens. To build without installing:

```sh
cargo build --release
./target/release/opencode-token-dashboard          # Linux/macOS
target\release\opencode-token-dashboard.exe       # Windows
```

### Options

```
opencode-token-dashboard [--web] [--host ADDR] [--port PORT] [--db-path PATH]
```

| Option | Meaning |
|---|---|
| *(no flag)* | Open the native GUI window (default) |
| `--web` | Run the local web server instead of the GUI |
| `--host` | Web server bind address (default `127.0.0.1`) |
| `--port` | Web server port (default `3000`, or `PORT`) |
| `--db-path` | Explicit database path (or `OPENCODE_DB_PATH`) |

## Terminal commands

For scripts and cron, the same data is available without opening a window:

```sh
opencode-token-dashboard report                       # human-readable summary
opencode-token-dashboard export --out /tmp/exports    # sessions + daily CSV
```

Both accept `--db-path PATH`. `report` prints today, all-time, yesterday, the 7-day average, the busiest day and the top model. `export` writes `opencode-token-usage-sessions.csv` and `opencode-token-usage-daily.csv` into the chosen directory (created if needed).

## Local JSON API

With `--web`, the same data is available as JSON under `/api/v1`:

- `GET /usage` — full snapshot
- `GET /usage/daily` — per-day rows
- `GET /usage/sessions` — searchable, sortable sessions
- `GET /usage/costs` — cost estimates
- `GET /settings`, `PUT /settings` — local UI settings
- `GET /health/live`, `GET /health/ready` — health checks

The API binds to `127.0.0.1` and has no authentication. Do not expose it to an untrusted network. Full contract and curl examples: [API.md](API.md) and [openapi.yaml](openapi.yaml).

## Auto sync

The dashboard keeps itself up to date with the OpenCode database:

- **Native GUI:** watches the database, WAL and shared-memory files and reloads when they change. The **Auto sync** selector offers Off, 1, 5, 15 or 60 seconds; the **Last synced** label shows when it last read the database. The interval and the chart range are saved between runs. Settings live in `~/.config/opencode-token-dashboard/config.json` (platform-specific, overridable with `OTD_CONFIG_DIR`).
- **Web mode:** pages reload every 15 seconds via a pause-aware timer that stops while the tab is hidden or an input is focused. Set `OTD_AUTO_REFRESH_SECONDS=0` to disable, or another number to change the interval. Auto-refresh is disabled on `/sessions` while a search is active so your input is not wiped.
- Web requests reuse the last snapshot while the database files are unchanged, so repeated reads are fast.

Manual **Refresh** is always available in both modes.

## Database location

The app checks, in order:

1. `OPENCODE_DB_PATH` / `--db-path`
2. Windows: `%LOCALAPPDATA%\opencode\opencode.db`, then `%APPDATA%\opencode\opencode.db`
3. macOS: `~/Library/Application Support/opencode/opencode.db`
4. Linux and fallback: `~/.local/share/opencode/opencode.db`

If OpenCode stores its database somewhere else on your machine, set `OPENCODE_DB_PATH`.

## The native app

- **Overview** — totals, an insights card (spike vs 7-day average, busiest day, cache share, top model), a custom-drawn chart, daily detail table, top sessions.
- **Sessions** — live search, sortable columns, CSV export (saved to the current directory).
- **Models / Projects** — usage tables with share bars.
- **Costs** — estimates from your own `pricing.json`; hidden until you supply rates.
- **Refresh** reloads the database without restarting. Startup is non-blocking: the window opens immediately and fills when the data is ready.

All UI is drawn with egui. The chart is painted with egui shapes; there is no HTML, CSS or JavaScript in the native window.

## Web mode

```sh
opencode-token-dashboard --web
```

Serves the same dashboard at <http://127.0.0.1:3000> using the original website design. CI builds and tests the Rust code on Ubuntu, Windows and macOS, and uploads a binary for each platform.

## Gentoo

```sh
emerge --ask dev-lang/rust
cargo install --path .
opencode-token-dashboard
```

SQLite is bundled, so `dev-db/sqlite` is not required. An OpenRC service file for `--web` mode is in `contrib/opencode-token-dashboard.initd`:

```sh
sudo cp contrib/opencode-token-dashboard.initd /etc/init.d/opencode-token-dashboard
sudo chmod +x /etc/init.d/opencode-token-dashboard
rc-update add opencode-token-dashboard default
rc-service opencode-token-dashboard start
```

## Security

- Database is opened read-only; a test proves an INSERT fails.
- Only token counters and labels are read — no prompt text or message content.
- All SQL is static; the search filters in memory.
- No network calls, telemetry, cookies or secrets.
- Web mode sets nosniff, DENY framing, no-referrer and permissions-policy headers.

## Tests

```sh
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The 10 Rust tests cover formatting, CSV escaping, chart generation, read-only database access and aggregation against a synthetic fixture database. Optional browser E2E for web mode lives in `legacy-nextjs/e2e-rust.mjs`.

## Notes

- "Fresh" means input + output + reasoning; "Total" includes cache reads and writes, which usually dominate.
- Days use the machine's local calendar date.
- If you expose web mode beyond localhost, add authentication and network controls.

## Rollback

```sh
git revert <commit>
```
