# Test report — opencode-token-dashboard

Date: 2026-11-18. Environment: local Next.js dev server on `localhost:3000`, Node 26, Chromium (Playwright 1.65 headless shell and the in-app browser).

## What ran

| Check | Command | Result |
|---|---|---|
| Unit + fixture-DB integration tests | `npm test` | 14 passed (2 files) |
| Type check | `npx tsc --noEmit` | pass |
| Lint | `npm run lint` | pass |
| Production build | `npm run build` | pass, 4 routes |
| Design lint | `node <hallmark>/scripts/sloplint.mjs src --scope app --genre modern-minimal --url http://localhost:3000/` | 0 FAIL, 0 WARN, 2 WAIVED (7b: declared absolute dark surface) |
| E2E + responsive | `npm run test:e2e` | 24 page loads (4 routes × 6 widths), 0 failures |
| Lighthouse | in-app browser Lighthouse on `/` | accessibility 1.00, best practices 1.00, SEO 1.00, 0 axe failures |
| Internal links | Playwright crawl of all `a[href]` | 14 links checked, 0 broken |
| Error state | `next start` with `OPENCODE_DB_PATH` pointing at a missing file | error boundary shows "Could not load token usage" and a working "Try again" button |

## E2E coverage
- Every route at 360, 390, 768, 1024, 1440 and 1920px: heading present, HTTP 200, no horizontal document scroll.
- Search `?q=greeting` returns 4 matching sessions.
- Sort `?sort=input&dir=asc` applies and renders the expected column.
- Chart range `?range=all` renders the stats and chart.
- No console errors or uncaught exceptions on any page.
- Screenshots saved under `test-results/` (gitignored) at 390px and 1440px and inspected by eye.

## Bugs found and fixed during testing
1. `/models` and `/projects` logged a Cache Components instant-navigation error to the browser console. Fixed with `export const instant = false` on every page (data is intentionally live).
2. At 390px the top-bar navigation and Refresh button were clipped. Fixed by making the nav horizontally scrollable, allowing the page head to wrap, and preventing the wordmark from wrapping.
3. The sticky table header overlapped the first row inside the table's scroll container. Fixed by removing the viewport sticky offset; the top bar was made static so only one sticky element remains (also satisfies Hallmark gate 56).
4. Lighthouse/axe found a label/name mismatch on the Refresh button (visible text "Refresh", aria-label "Reload token usage from the database"). Removed the redundant aria-label.
5. The chart areas appeared blank in the first screenshot because Recharts was still animating; confirmed correct after animation completed. No code change needed.

## Not run (with reasons)
- Firefox and WebKit: only Chromium is installed in this environment. The app uses standard React/CSS with no browser-specific APIs; cross-browser verification is still open.
- 200% browser zoom and reduced-motion emulation: not available through the test tooling here. CSS uses a `prefers-reduced-motion` fallback and a fluid type scale, but explicit emulation remains open.
- OWASP ZAP dynamic scan: no public preview deployment exists; the app is local-only.
- Semgrep: engine crashed in this environment (exit -11), documented in SECURITY-REVIEW.md.

## How to re-run
```sh
npm test
npm run lint
npx tsc --noEmit
npm run build
npm run dev
npm run test:e2e
```
