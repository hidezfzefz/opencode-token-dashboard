# opencode-token-dashboard

A native Rust web app that reads OpenCode's SQLite database and shows token usage: today and all-time totals, per-day history, and breakdowns by session, model and project. It keeps the same website as the original implementation, but runs as a single small binary with **no Node.js runtime, npm, or network access required**.

The previous Next.js implementation is preserved in `legacy-nextjs/` for reference. It is not needed to build or run the app.

## Requirements

- Rust (stable; a recent version is needed for the crates used here)
- A C compiler (`cc`) — `rusqlite` builds a bundled SQLite, so no separate SQLite development package is needed
- An existing OpenCode database, normally `~/.local/share/opencode/opencode.db`

Nothing else. No Node, no npm, no database server.

## Build and run

```sh
cargo build --release
./target/release/opencode-token-dashboard
```

Then open <http://127.0.0.1:3000>.

### Options

```sh
opencode-token-dashboard [--host ADDRESS] [--port PORT] [--db-path PATH]
```

| Option | Default |
|---|---|
| `--host` | `127.0.0.1` |
| `--port` | `3000` (or `PORT` env var) |
| `--db-path` | `~/.local/share/opencode/opencode.db` (or `OPENCODE_DB_PATH` env var) |

`--help` prints the same information. The database is always opened with `SQLITE_OPEN_READ_ONLY`.

## Gentoo

Install a Rust toolchain (a C compiler is already part of a normal Gentoo system):

```sh
emerge --ask dev-lang/rust
```

Build and install the binary:

```sh
git clone https://github.com/hidezfzefz/opencode-token-dashboard.git
cd opencode-token-dashboard
cargo build --release
install -m 755 target/release/opencode-token-dashboard /usr/local/bin/
```

Run it as your normal user so it can read your OpenCode database:

```sh
OPENCODE_DB_PATH=/home/youruser/.local/share/opencode/opencode.db \
  /usr/local/bin/opencode-token-dashboard --host 127.0.0.1 --port 3000
```

An OpenRC service file is included at `contrib/opencode-token-dashboard.initd`:

```sh
cp contrib/opencode-token-dashboard.initd /etc/init.d/opencode-token-dashboard
chmod +x /etc/init.d/opencode-token-dashboard
# edit OPENCODE_DB_PATH and command_args in the file, then:
rc-update add opencode-token-dashboard default
rc-service opencode-token-dashboard start
```

SQLite is bundled into the binary (`rusqlite` with the `bundled` feature), so you do **not** need `dev-db/sqlite`. The release binary is about 4 MB and is fully self-contained apart from glibc.

## Pages

- `/` — totals, yesterday and 7-day average, daily SVG chart, daily detail table, top sessions
- `/sessions` — searchable, sortable session table with CSV export
- `/models`, `/projects` — usage breakdowns with share bars
- `/daily.csv`, `/sessions.csv` — CSV downloads
- `/icon.svg`, `/static/style.css` — embedded assets

All pages are rendered server-side with Tera templates compiled into the binary. The chart is generated as SVG in Rust; there is no JavaScript in the app.

## Security

- The database connection is read-only; a unit test proves an INSERT fails.
- No prompt text or message content is read or rendered — only token counters and labels.
- All SQL is static; query parameters are only used for in-memory filtering and sorting.
- Security headers are set on every response: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`.
- No network calls, no telemetry, no secrets.

## Tests

```sh
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

`cargo test` covers number formatting, CSV escaping, the SVG chart, read-only database access, and aggregation against a synthetic fixture database. Optional browser E2E (needs the archived Node toolchain):

```sh
cd legacy-nextjs
npm ci
node e2e-rust.mjs
```

CI runs formatting, clippy, tests and a release build on every push and pull request.

## Notes

- Days are grouped by the machine's local calendar date.
- "Fresh" means input + output + reasoning. "Total" includes cache reads and writes, which usually dominate.
- The app is designed for localhost. If you expose it beyond your own machine, put it behind authentication and a network control.

## Rollback

The project uses git. To undo a change:

```sh
git revert <commit>
```
