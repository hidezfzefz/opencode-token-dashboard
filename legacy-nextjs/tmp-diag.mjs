import { chromium } from "playwright";
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [], failures = [];
page.on("console", m => { if (m.type() === "error") errors.push(m.text()); });
page.on("pageerror", e => errors.push("pageerror: " + e.message));
page.on("requestfailed", r => failures.push(r.url() + " " + r.failure()?.errorText));
page.on("crash", () => errors.push("PAGE CRASHED"));
await page.goto("http://localhost:3000/", { waitUntil: "networkidle" });
await page.waitForTimeout(1500);
const info = await page.evaluate(() => ({
  title: document.title,
  h1: document.querySelector("h1")?.innerText,
  stats: [...document.querySelectorAll(".stat-value")].map(e => e.textContent),
  svg: document.querySelectorAll("svg").length,
  cssLoaded: getComputedStyle(document.body).backgroundColor,
  bodyTextLength: document.body.innerText.length,
}));
await page.screenshot({ path: "/tmp/diag.png" });
console.log(JSON.stringify({ info, errors, failures }, null, 2));
await browser.close();
