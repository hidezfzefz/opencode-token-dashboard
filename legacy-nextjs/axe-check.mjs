import { chromium } from "playwright";
import AxeBuilder from "@axe-core/playwright";

const BASE = process.env.BASE_URL || "http://localhost:3002";
const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await context.newPage();
const violations = [];

for (const route of ["/", "/sessions", "/models", "/projects"]) {
  await page.goto(BASE + route, { waitUntil: "networkidle" });
  const results = await new AxeBuilder({ page }).analyze();
  for (const violation of results.violations) {
    violations.push(`${route}: ${violation.id} (${violation.impact})`);
  }
}

console.log(JSON.stringify({ violations }, null, 2));
await browser.close();
process.exit(violations.length ? 1 : 0);
