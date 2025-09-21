import { Page } from "playwright";
import { ManagedBrowserPage } from "../playwright.ts";
import { sleep } from "../time.ts";
import { assertExists } from "@std/assert";

export async function scrapEtf(
  id: string,
  region: Region,
  page: ManagedBrowserPage,
): Promise<EtfMetric> {
  await page.goto(region.productUrl(id));
  return {
    longTermTotalReturn: await scrapLongTermTotalReturn(page.page),
  };
}

export interface EtfMetric {
  longTermTotalReturn: number;
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

export const UnitedStatesRegion: Region = {
  code: "United States",
  productUrl(id: string): string {
    return `https://ishares.com/us/products/${id}/`;
  },
};

export const JapanRegion: Region = {
  code: "日本",
  productUrl(id: string): string {
    return `https://blackrock.com/jp/individual-en/en/products/${id}/`;
  },
};

export abstract class Region {
  abstract productUrl(id: string): string;
  abstract code: string;

  private static codeMap = new Map([
    [UnitedStatesRegion.code, UnitedStatesRegion],
    [JapanRegion.code, JapanRegion],
  ]);

  static parse(code: string): Region | undefined {
    if (code.length === 0) {
      return undefined;
    }

    const region = this.codeMap.get(code);
    assertExists(region, `Unknown region ${code}`);
    return region;
  }
}
