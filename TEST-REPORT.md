# Test report — opencode-token-dashboard

Date: 2026-11-18. Environment: local Next.js dev server on `localhost:3000`, Node 26, Chromium (Playwright 1.65 headless shell and the in-app browser).

## What ran

| Check | Command | Result |
|---|---|---|
| Unit + fixture-DB integration tests | `npm test` | 18 passed (3 files) |
| Type check | `npx tsc --noEmit` | pass |
| Lint | `npm run lint` | pass |
| Production build | `npm run build` | pass, 4 routes |
| Design lint | `node <hallmark>/scripts/sloplint.mjs src --scope app --genre modern-minimal --url http://localhost:3000/` | 0 FAIL, 0 WARN, 2 WAIVED (7b: declared absolute dark surface) |
| E2E + responsive | `npm run test:e2e` | 48 page loads (4 routes × 6 widths × 2 browsers), 0 failures |
| Cross-browser | `BROWSERS=chromium,firefox npm run test:e2e` | Chromium and Firefox both pass, including CSV export, zoom and reduced motion |
| Lighthouse | in-app browser Lighthouse on `/` | accessibility 1.00, best practices 1.00, SEO 1.00, 0 axe failures |
| Internal links | Playwright crawl of all `a[href]` | 14 links checked, 0 broken |
| Error state | `next start` with `OPENCODE_DB_PATH` pointing at a missing file | error boundary shows "Could not load token usage" and a working "Try again" button |

## E2E coverage
- Every route at 360, 390, 768, 1024, 1440 and 1920px: heading present, HTTP 200, no horizontal document scroll.
- Search `?q=greeting` returns 4 matching sessions.
- Sort `?sort=input&dir=asc` applies and renders the expected column.
- Chart range `?range=all` renders the stats and chart.
- No console errors or uncaught exceptions on any page.
- CSV export produces a header plus one row per visible session, with correct escaping and ISO timestamps.
- 200% zoom approximation (CSS zoom 2 at a 1440px viewport) produces no horizontal page scroll.
- `prefers-reduced-motion: reduce` disables the animation on a probe element.
- Screenshots saved under `test-results/` (gitignored) at 390px and 1440px and inspected by eye.

## Bugs found and fixed during testing
1. `/models` and `/projects` logged a Cache Components instant-navigation error to the browser console. Fixed with `export const instant = false` on every page (data is intentionally live).
2. At 390px the top-bar navigation and Refresh button were clipped. Fixed by making the nav horizontally scrollable, allowing the page head to wrap, and preventing the wordmark from wrapping.
3. The sticky table header overlapped the first row inside the table's scroll container. Fixed by removing the viewport sticky offset; the top bar was made static so only one sticky element remains (also satisfies Hallmark gate 56).
4. Lighthouse/axe found a label/name mismatch on the Refresh button (visible text "Refresh", aria-label "Reload token usage from the database"). Removed the redundant aria-label.
5. The chart areas appeared blank in the first screenshot because Recharts was still animating; confirmed correct after animation completed. No code change needed.

## Not run (with reasons)
- WebKit: the Playwright WebKit binary is installed, but this machine lacks the host libraries it needs (`libicu*.so.74`, `libsystemd.so.0`, `libxml2.so.2`, `libgstcodecparsers-1.0.so.0`, `libharfbuzz-icu.so.0`, `libjpeg.so.8`, `libmanette-0.2.so.0`, `libenchant-2.so.2`). Installing them needs root access, which is not available here. Chromium and Firefox both pass the full suite.
- OWASP ZAP dynamic scan: no public preview deployment exists; the app is local-only.
- Semgrep: engine crashed in this environment (exit -11), documented in SECURITY-REVIEW.md.

## How to re-run
```sh
npm test
npm run lint
npx tsc --noEmit
npm run build
npm run dev
npm run test:e2e                              # Chromium
BROWSERS=chromium,firefox npm run test:e2e     # both working browsers
```
