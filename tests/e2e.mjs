import { chromium } from "playwright";
import { mkdirSync } from "node:fs";

const BASE = process.env.BASE_URL || "http://localhost:3000";
const WIDTHS = [360, 390, 768, 1024, 1440, 1920];
const ROUTES = ["/", "/sessions", "/models", "/projects"];
const failures = [];
const notes = [];

mkdirSync("test-results", { recursive: true });

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext();
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
    if (!response || !response.ok()) failures.push(`${width}px ${route}: HTTP ${response?.status()}`);
    const heading = await page.locator("h1").first().innerText().catch(() => "");
    if (!heading) failures.push(`${width}px ${route}: no h1`);
    const scroll = await page.evaluate(() => ({
      scrollWidth: document.documentElement.scrollWidth,
      innerWidth: window.innerWidth,
    }));
    if (scroll.scrollWidth > scroll.innerWidth + 1) {
      failures.push(`${width}px ${route}: horizontal scroll ${scroll.scrollWidth} > ${scroll.innerWidth}`);
    }
    if (width === 390 || width === 1440) {
      await page.screenshot({ path: `test-results/${route === "/" ? "overview" : route.slice(1)}-${width}.png`, fullPage: false });
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
if (rows < 1) failures.push("search: no rows for q=greeting");
notes.push(`search q=greeting returned ${rows} rows`);

await page.goto(BASE + "/sessions?sort=input&dir=asc", { waitUntil: "networkidle" });
const firstInput = await page.locator("tbody tr td:nth-child(3)").first().innerText();
notes.push(`sort=input&dir=asc first input cell: ${firstInput}`);

await page.goto(BASE + "/?range=all", { waitUntil: "networkidle" });
if (!(await page.getByText("All time, all tokens").count())) failures.push("range=all: missing stats");

if (consoleErrors.length) failures.push(`console errors: ${consoleErrors.slice(0, 3).join(" | ")}`);
else notes.push("no console errors across all pages");

await browser.close();

console.log(JSON.stringify({ failures, notes }, null, 2));
process.exit(failures.length ? 1 : 0);
