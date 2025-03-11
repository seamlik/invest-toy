import { StockMetric } from "../../json-schema/typescript";
import { navigateToBlobInTab } from "./blob/execute";

type MessageListener = (
  message: unknown,
  sender: chrome.runtime.MessageSender,
  sendResponse: () => void,
) => boolean | undefined;

const stockMetricMessageTimeoutInMillisesonds = 4000;
const stockMetricMessageListeners: MessageListener[] = [];

export async function generateReport() {
  const tabIdPortfolio = await visitPortfolio();

  const urls = await queryInTab(tabIdPortfolio, queryAllStockUrls);
  console.info(`Found ${urls.length.toString()} stocks`);

  const metrics: StockMetric[] = [];
  try {
    for (const url of urls) {
      const metric = await extractStockMetric(url);
      metrics.push(metric);
    }
  } finally {
    stockMetricMessageListeners.forEach((listener) => {
      chrome.runtime.onMessage.removeListener(listener);
    });
  }

  await navigateToBlobInTab(tabIdPortfolio, metrics);
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
  const metricPromise = receiveStockMetricFromTab(tabId);
  console.info(`Extracting stock metric from: ${url}`);
  await executeInTab(tabId, "extract-stock-metric.js");
  const metric = await metricPromise;
  await chrome.tabs.remove(tabId);
  return metric;
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

async function executeInTab(tabId: number, script: string) {
  await chrome.scripting.executeScript({
    target: { tabId: tabId },
    files: [script],
  });
}

async function receiveStockMetricFromTab(tabId: number): Promise<StockMetric> {
  return new Promise<StockMetric>((resolve, reject) => {
    setTimeout(() => {
      reject(new Error(`Timeout awaiting stock metric`));
    }, stockMetricMessageTimeoutInMillisesonds);
    const listener: MessageListener = (message, sender, _) => {
      if (tabId === sender.tab?.id) {
        resolve(message as StockMetric);
      }
      return true;
    };
    chrome.runtime.onMessage.addListener(listener);
    stockMetricMessageListeners.push(listener);
  });
}
