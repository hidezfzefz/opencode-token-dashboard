import { chromium, firefox, webkit } from "playwright";
import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const BASE = process.env.BASE_URL || "http://localhost:3000";
const BROWSERS = (process.env.BROWSERS || "chromium").split(",");
const ENGINES = { chromium, firefox, webkit };
const WIDTHS = [360, 390, 768, 1024, 1440, 1920];
const ROUTES = ["/", "/sessions", "/models", "/projects"];
const failures = [];
const notes = [];

mkdirSync("test-results", { recursive: true });

for (const name of BROWSERS) {
  const engine = ENGINES[name];
  if (!engine) {
    failures.push(`unknown browser engine: ${name}`);
    continue;
  }
  const browser = await engine.launch({ headless: true });
  const context = await browser.newContext({ acceptDownloads: true });
  const page = await context.newPage();
  const consoleErrors = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });
  page.on("pageerror", (error) => consoleErrors.push(error.message));

  for (const width of WIDTHS) {
    await page.setViewportSize({ width, height: 900 });
    for (const route of ROUTES) {
      const response = await page.goto(BASE + route, { waitUntil: "networkidle" });
      if (!response || !response.ok()) failures.push(`${name} ${width}px ${route}: HTTP ${response?.status()}`);
      const heading = await page.locator("h1").first().innerText().catch(() => "");
      if (!heading) failures.push(`${name} ${width}px ${route}: no h1`);
      const scroll = await page.evaluate(() => ({
        scrollWidth: document.documentElement.scrollWidth,
        innerWidth: window.innerWidth,
      }));
      if (scroll.scrollWidth > scroll.innerWidth + 1) {
        failures.push(`${name} ${width}px ${route}: horizontal scroll ${scroll.scrollWidth} > ${scroll.innerWidth}`);
      }
      if (name === "chromium" && (width === 390 || width === 1440)) {
        const label = route === "/" ? "overview" : route.slice(1);
        await page.screenshot({ path: join("test-results", `${label}-${width}.png`), fullPage: false });
      }
    }
  }

  await page.setViewportSize({ width: 1440, height: 900 });

  await page.goto(BASE + "/sessions", { waitUntil: "networkidle" });
  const search = page.getByLabel("Search sessions");
  await search.fill("greeting");
  await search.press("Enter");
  await page.waitForURL(/q=greeting/);
  const rows = await page.locator("tbody tr").count();
  if (rows < 1) failures.push(`${name} search: no rows for q=greeting`);
  notes.push(`${name}: search q=greeting returned ${rows} rows`);

  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Export CSV" }).click(),
  ]);
  const path = await download.path();
  const csv = readFileSync(path, "utf8");
  const csvLines = csv.trim().split("\n");
  if (!csvLines[0].startsWith("Session,Last active,Model")) failures.push(`${name} export: bad CSV header`);
  if (csvLines.length !== rows + 1) failures.push(`${name} export: ${csvLines.length - 1} rows, expected ${rows}`);
  notes.push(`${name}: CSV export contained ${csvLines.length - 1} rows`);

  await page.goto(BASE + "/sessions?sort=input&dir=asc", { waitUntil: "networkidle" });
  const firstInput = await page.locator("tbody tr td:nth-child(3)").first().innerText();
  notes.push(`${name}: sort=input&dir=asc first input cell: ${firstInput}`);

  await page.goto(BASE + "/?range=all", { waitUntil: "networkidle" });
  if (!(await page.getByText("All time, all tokens").count())) failures.push(`${name} range=all: missing stats`);

  if (consoleErrors.length) failures.push(`${name} console errors: ${consoleErrors.slice(0, 2).join(" | ")}`);
  else notes.push(`${name}: no console errors across all pages`);

  // 200% zoom approximation: double the CSS zoom at a 1440px viewport.
  await page.goto(BASE + "/", { waitUntil: "networkidle" });
  const zoom = await page.evaluate(() => {
    document.documentElement.style.zoom = "2";
    return { scrollWidth: document.documentElement.scrollWidth, innerWidth: window.innerWidth };
  });
  notes.push(`${name}: 200% zoom doc width ${zoom.scrollWidth} / ${zoom.innerWidth}`);
  if (zoom.scrollWidth > zoom.innerWidth + 1) failures.push(`${name}: horizontal scroll at 200% zoom`);
  await page.evaluate(() => (document.documentElement.style.zoom = ""));

  await page.emulateMedia({ reducedMotion: "reduce" });
  const motion = await page.evaluate(() => {
    const probe = document.createElement("div");
    probe.className = "spin";
    document.body.appendChild(probe);
    const name = getComputedStyle(probe).animationName;
    probe.remove();
    return name;
  });
  if (motion !== "none") failures.push(`${name}: reduced motion still animates (${motion})`);
  else notes.push(`${name}: prefers-reduced-motion disables animation`);
  await page.emulateMedia({ reducedMotion: "no-preference" });

  await browser.close();
}

console.log(JSON.stringify({ browsers: BROWSERS, failures, notes }, null, 2));
process.exit(failures.length ? 1 : 0);
