# opencode-token-dashboard

A local-first dashboard for [OpenCode](https://opencode.ai) token usage. It reads OpenCode's own SQLite database and shows today's and all-time totals, a daily usage chart, and breakdowns by session, model and project.

Nothing is uploaded, no account is needed, and the database is opened read-only.

## Requirements

- Node.js 22 or newer (uses the built-in `node:sqlite` module; developed on Node 26)
- npm
- An existing OpenCode database, normally at `~/.local/share/opencode/opencode.db`

## Run it locally

```sh
npm install
npm run dev
```

Open <http://localhost:3000>.

For a production-style local run:

```sh
npm run build
npm start
```

### Configuration

| Variable | Default | Purpose |
|---|---|---|
| `OPENCODE_DB_PATH` | `~/.local/share/opencode/opencode.db` | Path to the OpenCode SQLite database to read |

Example:

```sh
OPENCODE_DB_PATH=/path/to/opencode.db npm run dev
```

There are no secrets and no `.env` file is required. The database is always opened with `readOnly: true`.

## What it shows

- **Overview** — today and all-time totals, fresh vs cached tokens, a daily stacked chart (last 30 days or all time), and the top sessions.
- **Sessions** — every session with input, output, reasoning, cache read and total, searchable by title/model/project, sortable by any column, and exportable as CSV.
- **Models** — totals and share of usage per model and variant.
- **Projects** — totals and share of usage per project directory.

"Fresh" means input + output + reasoning. "Total" also includes cache reads and cache writes. Cache reads are usually the largest number and are always labelled separately so the figures are not misleading.

Days are grouped by the machine's local calendar date.

## Tests and checks

```sh
npm test                    # Vitest unit + fixture-DB integration tests
npm run test:e2e            # Playwright E2E and responsive checks (requires a running server)
npm run lint                # ESLint
npx tsc --noEmit            # TypeScript
npm run build               # Production build
```

`npm run test:e2e` needs a dev server on `http://localhost:3000` and a Playwright browser (`npx playwright install chromium`). It checks four routes at six viewport widths, search, sorting, chart ranges, CSV export, 200% zoom, reduced motion and console errors. Use `BROWSERS=chromium,firefox npm run test:e2e` to run it in both browsers. WebKit needs system libraries that must be installed as root (`npx playwright install-deps webkit`).

See [TEST-PLAN.md](TEST-PLAN.md), [TEST-REPORT.md](TEST-REPORT.md) and [SECURITY-REVIEW.md](SECURITY-REVIEW.md) for the full results and open items.

## Design and data

- Next.js 16 App Router, React 19, TypeScript, Tailwind CSS v4, Recharts, Lucide.
- Convio Design tokens: dark, calm, minimal, all colours declared in one `:root` block in `src/app/globals.css`.
- All aggregation lives in `src/lib/usage.ts`; the read-only connection lives in `src/lib/db.ts`. No SQL is built from user input.
- No prompt text or message content is read or rendered — only token counters and labels.

## Deployment

There is intentionally no cloud deployment in v1. The app reads a local file, so it is meant to run on the same machine as OpenCode. If you expose it beyond localhost, add authentication and network controls first — see [SECURITY-REVIEW.md](SECURITY-REVIEW.md).

## Rollback

The project uses git. To undo a change, revert the relevant commit:

```sh
git revert <commit>
```

## Known limitations

- Chromium and Firefox pass the full E2E suite; WebKit could not run here because its host libraries are missing and installing them requires root.
- The chart animates on first load, so the first screenshot may show axes before the filled areas appear.
- Cache-read tokens dominate the raw totals; use the "fresh" figures when comparing actual model work.
- Development tooling has 16 high npm audit findings from an unpatched `picomatch` ReDoS advisory. Production dependencies have zero known vulnerabilities. See [SECURITY-REVIEW.md](SECURITY-REVIEW.md).
