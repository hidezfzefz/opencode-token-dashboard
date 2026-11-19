import { chromium, firefox } from "playwright";
const BASE = process.env.BASE_URL || "http://localhost:3000";
const failures = [];
const notes = [];
for (const name of ["chromium", "firefox"]) {
  const browser = await (name === "chromium" ? chromium : firefox).launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
  const consoleErrors = [];
  page.on("console", m => { if (m.type() === "error") consoleErrors.push(m.text()); });
  page.on("pageerror", e => consoleErrors.push(e.message));
  for (const width of [360, 390, 768, 1024, 1440, 1920]) {
    await page.setViewportSize({ width, height: 900 });
    for (const route of ["/", "/sessions", "/models", "/projects"]) {
      const res = await page.goto(BASE + route, { waitUntil: "networkidle" });
      if (!res.ok()) failures.push(`${name} ${width} ${route} HTTP ${res.status()}`);
      const scroll = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, i: window.innerWidth }));
      if (scroll.w > scroll.i + 1) failures.push(`${name} ${width} ${route} horizontal scroll ${scroll.w}>${scroll.i}`);
      if (name === "chromium" && (width === 390 || width === 1440)) {
        const label = route === "/" ? "overview" : route.slice(1);
        await page.screenshot({ path: `/tmp/rust-${label}-${width}.png` });
      }
    }
  }
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto(BASE + "/sessions", { waitUntil: "networkidle" });
  await page.getByLabel("Search sessions").fill("greeting");
  await page.getByLabel("Search sessions").press("Enter");
  await page.waitForURL(/q=greeting/);
  const rows = await page.locator("tbody tr").count();
  notes.push(`${name}: search=${rows}`);
  if (rows < 1) failures.push(`${name}: search empty`);
  await page.goto(BASE + "/sessions", { waitUntil: "networkidle" });
  const before = await page.locator("tbody tr").count();
  await page.locator('select[name="model"]').selectOption({ index: 1 });
  await page.getByRole("button", { name: "Apply filters" }).click();
  await page.waitForLoadState("networkidle");
  const after = await page.locator("tbody tr").count();
  if (!(after > 0 && after < before)) {
    failures.push(`${name}: model filter rows ${before} -> ${after}`);
  }
  notes.push(`${name}: model filter rows ${before} -> ${after}`);

  await page.goto(BASE + "/", { waitUntil: "networkidle" });
  await page.locator('a[href^="/sessions?q="]').first().click();
  await page.waitForURL(/\/sessions\?q=/);
  notes.push(`${name}: overview session link -> ${new URL(page.url()).search}`);

  const [d1] = await Promise.all([page.waitForEvent("download"), page.getByRole("link", { name: "Export CSV" }).click()]);
  notes.push(`${name}: sessions csv=${(await d1.suggestedFilename())}`);
  await page.goto(BASE + "/", { waitUntil: "networkidle" });
  const [d2] = await Promise.all([page.waitForEvent("download"), page.getByRole("link", { name: "Export daily CSV" }).click()]);
  notes.push(`${name}: daily csv=${(await d2.suggestedFilename())}`);
  if (consoleErrors.length) failures.push(`${name}: console ${consoleErrors.slice(0,2).join("|")}`);
  notes.push(`${name}: console errors=${consoleErrors.length}`);
  await browser.close();
}
console.log(JSON.stringify({ failures, notes }, null, 2));
process.exit(failures.length ? 1 : 0);
