# opencode-token-dashboard: brief

**What it is:** A local dashboard for OpenCode token usage, rewritten as a native Rust web app.
**For:** A developer running OpenCode on their own machine, including a Gentoo desktop. First action: open the dashboard and see today's and all-time token totals.
**Success:** The same website as before, served by one self-contained Rust binary that builds and runs on Gentoo with no Node.js runtime.

## Scope
- Pages: overview, sessions, models, projects, plus CSV downloads and a 404 page.
- Features: today/all-time totals, yesterday and 7-day average, daily SVG chart, daily detail table, search and sort, CSV export, read-only SQLite access.
- Not included: auth, cloud hosting, cost estimates, writing to the database, prompt content.

## Design
- The existing website is kept: Convio Design tokens, dark and calm, all colours in one CSS block.
- Server-rendered Tera templates and a Rust-generated SVG chart replace React/Recharts. No JavaScript.

## Stack
- Rust, Axum, Tokio, rusqlite (bundled SQLite), Tera, Chrono, Serde.
- No Node.js at runtime. The former Next.js app is archived under `legacy-nextjs/` for reference.

## Constraints
- Must build with just a Rust toolchain and a C compiler on Gentoo.
- Database is opened read-only; no data leaves the machine.
