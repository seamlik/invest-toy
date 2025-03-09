export async function generateReport() {
  const tabIdPortfolio = await visitPortfolio();
  const urls = await extractStockUrls(tabIdPortfolio);
  console.info(`Found ${urls.length.toString()} stocks`);
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
