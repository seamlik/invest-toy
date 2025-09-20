import * as ishares from "./scrapper/ishares.ts";
import * as playwright from "playwright";

const device = playwright.devices["Desktop Chrome"];
const channel = getBrowserChannel();

async function main(): Promise<void> {
  const browser = await playwright.chromium.launch({ channel: channel });
  try {
    const context = await browser.newContext({ ...device });
    try {
      context.setDefaultTimeout(0);

      const page = await context.newPage();
      try {
        const metrics = await ishares.scrapEtf("239686", page);
        console.info(metrics);
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
