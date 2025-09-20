import { Metric } from "../metric.ts";
import { Page } from "playwright";
import { navigate } from "../playwright.ts";
import { sleep } from "../time.ts";

export async function scrapEtf(
  id: string,
  page: Page,
): Promise<Map<Metric, number>> {
  await navigate(page, url(id));
  return new Map([
    [Metric.LongTermTotalReturn, await scrapLongTermTotalReturn(page)],
  ]);
}

function url(id: string): string {
  return `https://ishares.com/us/products/${id}/`;
}

async function scrapLongTermTotalReturn(page: Page): Promise<number> {
  const performanceTabs = page.locator("div#performanceTabs");

  console.info('Trying to click tab "Cumulative"');
  await page.getByRole("tab", { name: "Cumulative" }).click();

  console.info("Trying to select the most recent date");
  await performanceTabs
    .locator("div#cumulativeTabs select.date-dropdown")
    .selectOption({ index: 0 });
  await waitForPerformanceTableToReload();

  const totalReturnPercentage = await performanceTabs
    .locator("table.cumulative-returns > tbody > tr > td.fiveYear")
    .first()
    .innerText();
  return parseFloat(totalReturnPercentage) / 100;
}

async function waitForPerformanceTableToReload(): Promise<void> {
  await sleep(1000);
}
