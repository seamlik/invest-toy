import { StockMetric } from "./json-schema.js";
import { extractPriceChange } from "./metric/price-change/extract.js";

export async function generateReport() {
  const tabIdPortfolio = await visitPortfolio();

  const urls = await extractStockUrls(tabIdPortfolio);
  console.info(`Found ${urls.length.toString()} stocks`);

  await chrome.tabs.remove(tabIdPortfolio);

  const metrics: StockMetric[] = [];
  for (const url of urls) {
    const metric = await extractStockMetric(url);
    console.info(`Extracted: ${JSON.stringify(metric)}`);
    metrics.push(metric);
  }
}

async function visitPortfolio(): Promise<number> {
  const tabIdPortfolioList = await navigateTo(
    "https://finance.yahoo.com/portfolios",
  );
  const portfolioUrlResults = await chrome.scripting.executeScript({
    target: { tabId: tabIdPortfolioList },
    func: queryFirstPortfolioUrl,
  });
  const portfolioUrl = portfolioUrlResults[0].result as string;
  console.info(`Navigating to portfolio URL: ${portfolioUrl}`);
  await chrome.tabs.remove(tabIdPortfolioList);
  return await navigateTo(portfolioUrl);
}

function queryFirstPortfolioUrl(): string {
  const element = document.querySelector(
    'table[data-testid="table-container"] tbody tr td a.primary-link',
  );
  if (element instanceof HTMLAnchorElement) {
    return element.href;
  } else {
    throw new Error("Could not find the anchor to portfolio");
  }
}

async function extractStockUrls(tabId: number): Promise<string[]> {
  const stockUrlResults = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: queryAllStockUrls,
  });
  return stockUrlResults[0].result as string[];
}

function queryAllStockUrls(): string[] {
  const elements = document.querySelectorAll(
    "table > tbody > tr > td:first-child a",
  );
  const urls: string[] = [];
  for (const anchor of elements) {
    if (anchor instanceof HTMLAnchorElement) {
      urls.push(anchor.href);
    } else {
      throw new Error(`Element is not an anchor: ${anchor.getHTML()}`);
    }
  }
  return urls;
}

async function extractStockMetric(url: string): Promise<StockMetric> {
  const tabId = await navigateTo(url);

  const ticker = await queryInTab(tabId, queryTicker);
  if (ticker === null) {
    throw new Error(`Failed to extract ticker from: ${url}`);
  }

  const priceChangeInOneMonth =
    (await extractPriceChange(tabId, "1m")) ?? undefined;
  const priceChangeInFiveYears =
    (await extractPriceChange(tabId, "5y")) ?? undefined;
  const dividendYield: number | null = await executeInTab(
    tabId,
    "./metric/dividend-yield.js",
  );

  await chrome.tabs.remove(tabId);
  return {
    ticker: ticker,
    price_change_in_one_month: priceChangeInOneMonth,
    price_change_in_five_years: priceChangeInFiveYears,
    dividend_yield: dividendYield ?? undefined,
  };
}

function queryTicker(): string | null {
  const element = document.querySelector(
    'section[data-testid="quote-hdr"] div.hdr h1',
  );
  return element instanceof HTMLHeadingElement ? element.textContent : null;
}

async function navigateTo(url: string): Promise<number> {
  const tab = await chrome.tabs.create({
    url: url,
  });
  const tabId = tab.id;
  if (typeof tabId !== "number") {
    throw new Error("Tab has no ID");
  }
  return tabId;
}

async function queryInTab<T>(tabId: number, query: () => T): Promise<T> {
  const [result] = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: query,
  });
  return result.result as T;
}

async function executeInTab<T>(tabId: number, script: string): Promise<T> {
  const [result] = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    files: [script],
  });
  return result.result as T;
}
