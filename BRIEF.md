# opencode-token-dashboard: brief

**What it is:** A local-first web dashboard that reads OpenCode's own SQLite database and turns raw token counters into a clear picture of usage.
**For:** The developer running OpenCode on their own machine. First thing they do: open the dashboard and see today's and all-time token totals.
**Success:** In month one, the tool answers "how many tokens did I use today / ever / per project / per model" without hand-written SQL.

## Scope (v1)
- Pages or screens:
  - Overview — headline totals (today, all-time, fresh vs cached), per-day history chart, top consumers.
  - Sessions — sortable table of sessions with token columns, search by title.
  - Models — breakdown by model/provider.
  - Projects — breakdown by project/directory.
- Features:
  - Read `~/.local/share/opencode/opencode.db` read-only via `node:sqlite`, with configurable path via env var.
  - Aggregate tokens: input, output, reasoning, cache read, cache write.
  - "Fresh tokens" = input + output + reasoning; "Total" includes cache reads/writes.
  - Local timezone day bucketing; today = local calendar day.
  - Refresh button; no background workers.
- Not in v1: auth, hosted deployment, cost estimates, writing to the DB, prompt content display.

## Design
- Direction: Convio Design — dark, calm, minimal. All colors/spacing from CSS custom properties in one `:root` block. No framework default palette.
- Density-first dashboard: top bar, tabular numerals, tables before cards, 4 data states (loading/empty/error/full).
- Accessible: WCAG 2.2 AA, visible focus, prefers-reduced-motion.

## Stack
- Framework: Next.js (App Router, server components read SQLite directly).
- Styling: Tailwind CSS v4.
- Components: shadcn/ui-style local components + Lucide icons; Recharts for the history chart.
- Data: `node:sqlite` (built into Node 22+), read-only connection to the OpenCode DB. No hosted DB, no ORM, no auth.
- Testing: Vitest for aggregation logic with a synthetic fixture DB.
- Hosting: local-first (`npm run dev` / `npm run build && npm start`). Not deployed to a cloud host in v1.

## Content and integrations
- No external APIs. No telemetry. No network calls.

## Constraints and risks
- The OpenCode DB is live/WAL-mode; open read-only to avoid interference.
- Schema may evolve; aggregation code isolated in one module so it's easy to adapt.
- Cache-read tokens dominate raw totals; label them clearly to avoid misleading the user.

## Assumptions (please check)
- DB path is `~/.local/share/opencode/opencode.db` (overridable with `OPENCODE_DB_PATH`).
- "Today" uses the machine's local timezone.
- No credentials are needed because everything stays on the local machine.
