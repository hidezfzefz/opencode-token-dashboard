# opencode-token-dashboard: design

Direction: Convio Design (dark, calm, minimal), adapted to a density-first analytics dashboard. Hallmark's gates apply for layout, accessibility and anti-template rules; where Hallmark would deal a theme or use a hero, the project's Convio Design tokens and the app-screen rules win.

Set aside: Hallmark's theme draw and display-font reflex — Convio Design fixes Inter and the palette; this is a tool screen, not a marketing page, so there is no hero.

## Tokens (single `:root` block in `globals.css`)
Colors, radii, font and easing are copied from `convio/convio-design` tokens.css. Tailwind v4 `@theme` maps them to `bg-cv-card`, `text-cv-text-75`, `border-cv-line`, etc. No literal colors or font-family outside the token block.

- Surfaces: `--cv-bg` page, `--cv-card` / `--cv-card-2` panels, `--cv-raised` rows and inputs.
- Hairlines: `--cv-line`, stronger on focus.
- Text: `--cv-text`, `-75` body, `-50` secondary, `-25` placeholders only.
- Status: `--cv-ok`, `--cv-warn`, `--cv-bad` for text/dots/thin outlines only.
- Type scale: 40 / 28 / 20 / 16 / 14 / 12px, tabular numerals on figures.
- Spacing scale tokenized; cards 24px radius, rows/inputs 12–14px, pills 999px.
- Motion: 180–280ms transform/opacity only; `prefers-reduced-motion` disables all.

## Core components
- Top bar: wordmark "token usage" on the left, nav links (Overview, Sessions, Models, Projects), refresh action on the right. Collapses to a menu under 640px.
- Table first: dense rows, 32–40px control height with 44px hit area, sticky header, right-aligned tabular figures, hairline separators. No card-in-card.
- Stat: large number over a 12px caption. No icon tiles, no trend arrows, no decorative dots.
- Share bar: neutral second color, width reflects real share, `role="img"` with an aria-label.
- Every data region has four states: skeleton loading, empty with next action, error with recovery, and full.
- Focus: 2px `rgba(255,255,255,.6)` outline, 2px offset, instant.
- Chart: Recharts area/bar, axis labels from real data, no gradients, no invented series.

## Screen sketches
1. **Overview** — top bar; heading "Token usage"; a row of stats (today total, today fresh, all-time total, all-time fresh); a per-day stacked area chart (fresh vs cache) with a 30-day/all-time toggle in the URL; a top-five sessions table. Real numbers only; cache reads labelled clearly.
2. **Sessions** — search input (URL-backed `?q=`), sortable table: title, model, updated, input, output, reasoning, cache read, total. Empty state: "No sessions match this search." Error state: "Could not read the OpenCode database at <path>."
3. **Models** — table grouped by model/provider with totals and a share bar.
4. **Projects** — table grouped by project/directory with totals and a share bar.

## Screenshots and verification
- Check at 1280×800 and 360px: no horizontal scroll, nav usable, numbers readable.
- Run the Hallmark slopp lint against the built CSS before handover (if the script is available).
- Compare every displayed figure against a direct SQL query on the real DB.
