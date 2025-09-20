import * as scrapper from "./scrapper/market-stack.ts";
import * as playwright from "playwright";

const device = playwright.devices["Desktop Chrome"];
const channel = getBrowserChannel();

async function main(): Promise<void> {
  const browser = await playwright.chromium.launch({ channel: channel });
  try {
    const context = await browser.newContext({ ...device });
    try {
      const page = await context.newPage();
      try {
        console.info(
          await scrapper.scrapProducts([
            "NVDA",
            "1475.T",
            "2330.TW",
            "ASML.AS",
            "KOFOL.PR",
            "WISE.L",
          ]),
        );
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

await main();
