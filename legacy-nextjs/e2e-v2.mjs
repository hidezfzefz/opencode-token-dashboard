import { chromium, firefox, webkit } from "playwright";

const BASE = process.env.BASE_URL || "http://localhost:3002";
const ENGINES = { chromium, firefox, webkit };
const BROWSERS = (process.env.BROWSERS || "chromium,firefox,webkit").split(",");
const WIDTHS = [360, 390, 768, 1024, 1440, 1920];
const PAGES = ["/", "/sessions", "/models", "/projects"];
const APIS = [
  "/api/v1/usage",
  "/api/v1/usage/daily",
  "/api/v1/usage/sessions",
  "/api/v1/usage/costs",
  "/api/v1/settings",
  "/health/live",
  "/health/ready",
];
const failures = [];
const notes = [];

for (const name of BROWSERS) {
  const engine = ENGINES[name];
  let browser;
  try {
    browser = await engine.launch({ headless: true });
  } catch (error) {
    notes.push(`${name}: skipped (${String(error).split("\n")[0].slice(0, 120)})`);
    continue;
  }
  const context = await browser.newContext({ acceptDownloads: true });
  const page = await context.newPage();
  const consoleErrors = [];
  const failedRequests = [];
  page.on("console", (m) => { if (m.type() === "error") consoleErrors.push(m.text()); });
  page.on("pageerror", (e) => consoleErrors.push(e.message));
  page.on("requestfailed", (r) => {
    const error = r.failure()?.errorText || "";
    if (error.includes("ERR_ABORTED") && r.url().includes(".csv")) return;
    failedRequests.push(`${r.url()} ${error}`);
  });

  for (const width of WIDTHS) {
    await page.setViewportSize({ width, height: 900 });
    for (const route of PAGES) {
      const response = await page.goto(BASE + route, { waitUntil: "networkidle" });
      if (!response?.ok()) failures.push(`${name} ${width} ${route}: HTTP ${response?.status()}`);
      const heading = await page.locator("h1").first().innerText().catch(() => "");
      if (!heading) failures.push(`${name} ${width} ${route}: missing h1`);
      const scroll = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, i: window.innerWidth }));
      if (scroll.w > scroll.i + 1) failures.push(`${name} ${width} ${route}: horizontal scroll ${scroll.w}>${scroll.i}`);
      if (name === "chromium" && (width === 390 || width === 1440)) {
        const label = route === "/" ? "overview" : route.slice(1);
        await page.screenshot({ path: `/tmp/v2-${label}-${width}.png` });
      }
    }
  }

  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(BASE + "/sessions", { waitUntil: "networkidle" });
  await page.getByLabel("Search sessions").fill("greeting");
  await page.getByLabel("Search sessions").press("Enter");
  await page.waitForURL(/q=greeting/);
  const rows = await page.locator("tbody tr").count();
  if (rows < 1) failures.push(`${name}: search returned no rows`);
  const meta = await page.locator('meta[http-equiv="refresh"]').count();
  if (meta !== 0) failures.push(`${name}: auto-refresh active during search`);
  notes.push(`${name}: search rows=${rows}, refresh tags during search=${meta}`);

  for (const [label, name2] of [["Export CSV", "session"], ["Export daily CSV", "daily"]]) {
    await page.goto(BASE + (name2 === "session" ? "/sessions" : "/"), { waitUntil: "networkidle" });
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("link", { name: label }).click(),
    ]);
    notes.push(`${name}: ${name2} csv=${download.suggestedFilename()}`);
  }

  for (const api of APIS) {
    const response = await page.request.get(BASE + api);
    if (!response.ok()) failures.push(`${name} ${api}: HTTP ${response.status()}`);
    const body = await response.text();
    if (!body.trim().startsWith("{") && !body.trim().startsWith("[")) {
      failures.push(`${name} ${api}: not JSON`);
    }
  }
  notes.push(`${name}: ${APIS.length} API endpoints returned JSON`);

  if (consoleErrors.length) failures.push(`${name}: console errors ${consoleErrors.slice(0, 2).join("|")}`);
  if (failedRequests.length) failures.push(`${name}: failed requests ${failedRequests.slice(0, 2).join("|")}`);
  notes.push(`${name}: console errors=${consoleErrors.length}, failed requests=${failedRequests.length}`);
  await browser.close();
}

console.log(JSON.stringify({ failures, notes }, null, 2));
process.exit(failures.length ? 1 : 0);
