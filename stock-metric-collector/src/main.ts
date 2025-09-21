import * as playwright from "playwright";
import * as io from "@std/io";
import * as portfolio from "./portfolio.ts";

const device = playwright.devices["Desktop Chrome"];
const channel = getBrowserChannel();

async function main(): Promise<void> {
  const portfolio = await loadPortfolioFromStdIn();
  const browser = await playwright.chromium.launch({ channel: channel });
  try {
    const context = await browser.newContext({ ...device });
    try {
      const page = await context.newPage();
      try {
        console.info(portfolio);
      } finally {
        await page.close();
      }
    } finally {
      await context.close();
    }
  } finally {
    await browser.close();
  }
}

function getBrowserChannel(): string {
  const channelEnv = Deno.env.get("PLAYWRIGHT_BROWSER");
  return channelEnv === undefined ? "msedge" : channelEnv;
}

async function loadPortfolioFromStdIn(): Promise<portfolio.Product[]> {
  const csvTextBuffer = await io.readAll(Deno.stdin);
  const csvText = new TextDecoder("utf-8").decode(csvTextBuffer);
  return portfolio.load(csvText);
}

await main();
