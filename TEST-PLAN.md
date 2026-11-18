# opencode-token-dashboard: test plan

No sign-up, sign-in, roles, permissions or billing exist: this is a single-user local tool reading a local database. Those generic checks are not applicable and are replaced by read-only and data-integrity checks.

## Journeys

### 1. Overview
- Happy path: open `/`, see today and all-time totals, a daily chart, and the top sessions table. Figures match a direct SQL aggregate on the same DB.
- Edge: chart range `?range=all` shows all days; `?range=30d` shows the last 30.
- Edge: empty database yields an empty state, not a crash.
- Error: missing DB path yields the error boundary with a retry action and a clear message.

### 2. Sessions
- Happy path: open `/sessions`, see every session with token columns.
- Search: `?q=greeting` filters by session title, model or project; no match shows the empty state.
- Sort: each column header toggles direction and preserves the search query.
- Edge: unknown sort key falls back to total; unknown direction falls back to descending.

### 3. Models and projects
- Happy path: both tables render with real totals and share bars.
- Edge: no data shows the empty state.

### 4. Data integrity
- The DB handle is read-only: an INSERT throws.
- Token math: fresh = input + output + reasoning; total = fresh + cache read + cache write.
- Days use local calendar dates.

## Non-functional
- Unit tests: Vitest on aggregation, formatting and read-only enforcement.
- Browser: every journey in Chromium; no console errors or failed requests.
- Accessibility: keyboard navigation, visible focus, labelled form controls; Lighthouse accessibility score.
- Responsive: no horizontal page scroll from 360 to 1920px; tables scroll inside their own wrapper.
- Design lint: Hallmark sloplint app scope, 0 Floor failures.
- Links: all internal links resolve; no external links.

## Commands
- `npm test` — unit and fixture-DB integration tests
- `npm run lint` — ESLint
- `npx tsc --noEmit` — type check
- `npm run build` — production build
- `node <hallmark>/scripts/sloplint.mjs src --scope app --genre modern-minimal --url http://localhost:3000/` — design lint
