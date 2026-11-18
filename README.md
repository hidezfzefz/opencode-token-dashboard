# opencode-token-dashboard

A **fully native Rust desktop app** for OpenCode token usage. It opens as a real window on Linux, Windows and macOS — no browser, webview, Node.js or web server needed. It reads your OpenCode SQLite database read-only and shows today's and all-time totals, a daily chart, and breakdowns by session, model and project.

A browser version is still available with the `--web` flag, but the default is the native GUI.

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

## Database location

The app checks, in order:

1. `OPENCODE_DB_PATH` / `--db-path`
2. Windows: `%LOCALAPPDATA%\opencode\opencode.db`, then `%APPDATA%\opencode\opencode.db`
3. macOS: `~/Library/Application Support/opencode/opencode.db`
4. Linux and fallback: `~/.local/share/opencode/opencode.db`

If OpenCode stores its database somewhere else on your machine, set `OPENCODE_DB_PATH`.

## The native app

- **Overview** — totals, yesterday and 7-day average, a custom-drawn chart, daily detail table, top sessions.
- **Sessions** — live search, sortable columns, CSV export (saved to the current directory).
- **Models / Projects** — usage tables with share bars.
- **Refresh** reloads the database without restarting.

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
