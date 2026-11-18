# opencode-token-dashboard: design

The native window uses egui's immediate-mode widgets with a custom dark style set in `src/gui.rs`: near-black panels, hairline borders, muted secondary text and tabular-style figures.

## Layout
- Header: title, database path, Refresh button.
- Tab row: Overview, Sessions, Models, Projects.
- Overview: four stat cards, daily chart, daily detail table, top sessions.
- Sessions: search field, sort combo box, direction toggle, CSV export, striped scrollable grid.
- Models / Projects: striped grids with progress-bar share indicators.

## Chart
The chart is painted directly with egui shapes in `draw_chart`: grid lines, compact labels, a fresh-token area and a cache-read area stacked above it. It uses the same "nice step" axis calculation as the web SVG chart.

## Accessibility and cross-platform
- egui handles keyboard and pointer input natively on each platform.
- Error state explains how to set `OPENCODE_DB_PATH` or `--db-path`.
- CSV files are written to the current working directory with a clear status message.

## Web mode
The original Tera/CSS website is unchanged and remains available through `--web`.
