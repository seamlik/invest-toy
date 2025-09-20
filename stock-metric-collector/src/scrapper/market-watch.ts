import { Metric } from "../metric.ts";
import { Page } from "playwright";
import { parsePercentage } from "../number.ts";
import { navigate } from "../playwright.ts";

export async function scrapStock(
  ticker: string,
  page: Page,
): Promise<Map<Metric, number>> {
  await navigate(page, url(ticker));
  return new Map([
    [Metric.OneMonthPriceChange, await scrapOneMonthPriceChange(page)],
  ]);
}

function url(ticker: string): string {
  return `https://marketwatch.com/investing/stock/${ticker}`;
}

async function scrapOneMonthPriceChange(page: Page): Promise<number> {
  const oneMonthPriceChangeText = await page
    .locator("div#maincontent div.region--primary div.performance td li.value")
    .nth(1)
    .textContent();
  if (oneMonthPriceChangeText === null) {
    throw new Error("Price change not found");
  }

  return parsePercentage(oneMonthPriceChangeText);
}
