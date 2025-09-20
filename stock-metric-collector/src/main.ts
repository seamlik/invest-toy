import * as yahoo_finance from "./scrapper/yahoo-finance.ts";
import * as playwright from "playwright";

const device = playwright.devices["Desktop Chrome"];
const browser = await playwright.chromium.launch();
try {
  const context = await browser.newContext({ ...device });
  try {
    context.setDefaultTimeout(0);

    const page = await context.newPage();
    try {
      const metrics = await yahoo_finance.scrapStock("PYPL", page);
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
