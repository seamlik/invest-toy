import { Metric } from "../metric.ts";
import { Locator, Page } from "playwright";
import { parsePercentage } from "../number.ts";
import { ManagedBrowserPage } from "../playwright.ts";

export async function scrapStock(
  ticker: string,
  page: ManagedBrowserPage,
): Promise<Map<Metric, number>> {
  await page.goto(url(ticker));
  return new Map([
    [Metric.LongTermTotalReturn, await scrapLongTermTotalReturn(page.page)],
  ]);
}

function url(ticker: string): string {
  return `https://finance.yahoo.com/quote/${ticker}`;
}

async function scrapLongTermTotalReturn(page: Page): Promise<number> {
  const performanceOverviewSection = page.locator(
    'section[data-testid="performance-overview"]',
  );
  const cards = await performanceOverviewSection
    .locator('section[data-testid="card-container"]')
    .all();
  for (const card of cards) {
    if (await isFiveYear(card)) {
      return await scrapReturnFromPerformanceCard(card);
    }
  }
  throw new Error('"5-Year Return" not found');
}

async function isFiveYear(card: Locator): Promise<boolean> {
  return (await card.locator("header").innerText()) === "5-Year Return";
}

async function scrapReturnFromPerformanceCard(card: Locator): Promise<number> {
  const performanceDiv = card.locator("div.perf").first();
  const sign = await determineSign(performanceDiv);
  return parsePercentage(await performanceDiv.innerText()) * sign;
}

async function determineSign(performanceDiv: Locator): Promise<number> {
  const performanceDivClass = await performanceDiv.getAttribute("class");
  if (performanceDivClass === null) {
    throw new Error("Unable to determine if the sign of the long-term return");
  }

  const isPositive = performanceDivClass.search("positive") >= 0;
  const isNegative = performanceDivClass.search("negative") >= 0;
  if (isPositive && !isNegative) {
    return 1;
  } else if (!isPositive && isNegative) {
    return -1;
  } else {
    throw new Error("The sign of the long-term return is ambiguous");
  }
}
