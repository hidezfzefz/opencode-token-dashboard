# opencode-token-dashboard: build plan

## Architecture
- Next.js App Router. Server Components read SQLite directly via `node:sqlite` (read-only).
- Data layer in `src/lib/usage.ts`: one function opens the DB, one set of pure aggregation helpers.
- Pages: `/` overview, `/sessions`, `/models`, `/projects`. Each page calls the data layer server-side.
- Client components only for the refresh button and the chart (Recharts needs the browser).
- No API routes needed in v1; no auth; no writes to the OpenCode DB.

## Data model (read only)
- `session_message` (id, session_id, type, time_created, data JSON with `model`, `tokens`)
- `session_v2` (id, title, directory, project_id, time_created/updated, token columns, `model`)
- `project` (id, ...)
- Token fields: input, output, reasoning, cache.read, cache.write.
- Derived: fresh = input + output + reasoning; total = fresh + cache.read + cache.write.
- Day bucket = local timezone date from `time_created` (ms).

## Milestone 1 — Vertical slice: overview page
1. Scaffold Next.js + TypeScript + Tailwind v4; verify `npm run dev` serves a page. ✓ by curl HTTP 200.
2. Add design tokens in `src/app/globals.css` (:root block) and a top-bar layout. ✓ by screenshot at 1280×800 and 360px.
3. Implement `src/lib/usage.ts`: open DB read-only, aggregate today/all-time + per-day. ✓ by Vitest against a synthetic fixture DB.
4. Build overview page: headline totals, per-day chart, top sessions. ✓ by comparing numbers to a direct SQL query on the real DB.
5. Commit milestone 1.

## Milestone 2 — Breakdowns
6. `/sessions` table: search, sortable token columns, local timezone dates. ✓ Vitest + browser check.
7. `/models` and `/projects` breakdown tables with share bars. ✓ browser check on real data.
8. Refresh control that revalidates the server data. ✓ manual check.
9. Commit milestone 2.

## Milestone 3 — Quality and ship
10. Security review: read-only DB, no prompt content rendered, no network calls, path from env validated. ✓ checklist.
11. Tests: expand Vitest coverage; run `npm test`, `npm run lint`, `npm run build`. ✓ all green.
12. README with setup, env vars, screenshots description, and limitations.
13. Create GitHub repo and push; commit history with clear messages.

## Convio stage checkpoints
- [x] Discover — BRIEF.md written.
- [x] Plan — this file.
- [ ] Design — **call next_stage** before building any screen.
- [ ] Scaffold — **call next_stage** before creating the app.
- [ ] Data and auth — **call next_stage** before data-layer work.
- [ ] Build features — **call next_stage** before feature work.
- [ ] Security review — **call next_stage** before review.
- [ ] Test thoroughly — **call next_stage** before testing.
- [ ] Ship — **call next_stage** before handover.

## Notes
- The OpenCode DB is large (~260 MB) and in WAL mode; queries aggregate server-side, no raw rows shipped to the client.
- Cache reads dominate totals; label them explicitly so "fresh" vs "total" is never ambiguous.
