# opencode-token-dashboard: design

The website is preserved from the earlier implementation: dark, calm and minimal, with all colour and type tokens declared once in `static/style.css` (`:root` block). Tera templates in `templates/` reproduce the original layout.

## Layout
- Static top bar with wordmark and four navigation links. No sticky header, so the single sticky table header does not conflict.
- Dense, table-first dashboard with tabular numerals on figures.
- Cards use hairline borders and the existing radius tokens; no card-in-card.
- Four states where data loads: skeleton is not needed because rendering is synchronous; empty, error and full states are implemented.

## Components
- Stat, table, share bar, buttons, search input, chart — all styled with the existing CSS classes.
- The daily chart is an SVG generated in `src/chart.rs`. Two variants are embedded: a wide one for desktop and a narrow one for screens under 640px, so axis labels are not distorted by scaling.

## Accessibility
- `lang="en"`, labelled search input, visible focus styles from the token block, `prefers-reduced-motion` fallback, and ARIA labels on share bars and the chart.

## What changed from the Next.js version
- React/Recharts are gone. Pages and the chart are produced on the server.
- Refresh buttons are ordinary GET form submissions (the page always re-reads the database).
- The visual design, copy, tables and behaviour are unchanged.
