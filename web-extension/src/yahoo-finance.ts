export async function generateReport() {
  const tab = await chrome.tabs.create({
    url: "https://finance.yahoo.com/portfolios",
  });
  const tabId = tab.id;
  if (typeof tabId !== "number") {
    throw new Error("Active tab has no ID");
  }
  await visitPortfolio(tabId);
}

async function visitPortfolio(tabId: number) {
  const portfolioUrlResult = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: queryFirstPortfolioUrl,
  });
  const portfolioUrl = portfolioUrlResult[0].result as string;
  await chrome.tabs.update(tabId, { url: portfolioUrl });
}

function queryFirstPortfolioUrl(): string | null {
  const element = document.querySelector(
    'table[data-testid="table-container"] tbody tr td a.primary-link',
  );
  return element instanceof HTMLAnchorElement ? element.href : null;
}
