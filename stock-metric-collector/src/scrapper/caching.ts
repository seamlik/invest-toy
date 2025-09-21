import { ManagedBrowserPage } from "../playwright.ts";
import * as market_stack from "./market-stack.ts";
import * as yahoo_finance from "./yahoo-finance.ts";
import * as ishares from "./ishares.ts";
import { resolve } from "@std/path";

const today = Temporal.Now.plainDateISO();
const outputDirectoryPath = getOutputDirectoryPath();

function getOutputDirectoryPath(): string {
  const value = Deno.env.get("STOCK_METRIC_COLLECTOR_OUTPUT_DIRECTORY");
  return value ?? ".";
}

abstract class CachingScrapper<T> {
  protected abstract scrap(): Promise<T>;

  constructor(private metricsFilePath: string) {}

  async run(): Promise<T> {
    const metricsCached = await this.loadMetrics<T>(this.metricsFilePath);
    if (metricsCached !== undefined) {
      console.info(`Found cached result at ${this.metricsFilePath}`)
      return metricsCached;
    }

    const metrics = await this.scrap();
    await Deno.writeTextFile(this.metricsFilePath, JSON.stringify(metrics));
    return metrics;
  }

  private async loadMetrics<T>(filePath: string): Promise<T | undefined> {
    let text: string;
    try {
      text = await Deno.readTextFile(filePath);
    } catch (_e) {
      console.info(`Failed to read ${filePath}`);
      return undefined;
    }
    return JSON.parse(text) as T;
  }
}

export class CachingMarketStackScrapper extends CachingScrapper<
  market_stack.ProductMetric[]
> {
  constructor(private tickers: string[]) {
    super(resolve(outputDirectoryPath, `market-stack-${today}.json`));
  }

  protected override async scrap(): Promise<market_stack.ProductMetric[]> {
    return await market_stack.scrapProducts(this.tickers);
  }
}

export class CachingYahooFinanceStockScrapper extends CachingScrapper<yahoo_finance.StockMetric> {
  constructor(
    private ticker: string,
    private page: ManagedBrowserPage,
  ) {
    super(yahooFinanceMetricFilePath(ticker));
  }

  protected override async scrap(): Promise<yahoo_finance.StockMetric> {
    return await yahoo_finance.scrapStock(this.ticker, this.page);
  }
}

export class CachingYahooFinanceEtfScrapper extends CachingScrapper<yahoo_finance.EtfMetric> {
  constructor(
    private ticker: string,
    private page: ManagedBrowserPage,
  ) {
    super(yahooFinanceMetricFilePath(ticker));
  }

  protected override async scrap(): Promise<yahoo_finance.EtfMetric> {
    return await yahoo_finance.scrapEtf(this.ticker, this.page);
  }
}

function yahooFinanceMetricFilePath(ticker: string): string {
  return resolve(outputDirectoryPath, `yahoo-finance-${ticker}-${today}.json`);
}

export class CachingIsharesScrapper extends CachingScrapper<ishares.EtfMetric> {
  constructor(
    private ticker: string,
    private id: string,
    private region: ishares.Region,
    private page: ManagedBrowserPage,
  ) {
    super(resolve(outputDirectoryPath, `ishares-${ticker}-${today}.json`));
  }

  protected override async scrap(): Promise<ishares.EtfMetric> {
    return await ishares.scrapEtf(this.id, this.region, this.page);
  }
}
