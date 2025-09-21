import { ManagedBrowserPage } from "./playwright.ts";
import {
  CachingIsharesScrapper,
  CachingMarketStackScrapper,
  CachingYahooFinanceEtfScrapper,
  CachingYahooFinanceStockScrapper,
} from "./scrapper/caching.ts";
import * as market_stack from "./scrapper/market-stack.ts";
import { ProductMetric } from "../../json-schema/typescript/index.ts";
import { associateBy } from "@std/collections";
import { Region } from "./scrapper/ishares.ts";

export async function collectProductMetrics(
  portfolio: Product[],
): Promise<ProductMetric[]> {
  await using page = await ManagedBrowserPage.create(getBrowserChannel());
  const tickers = portfolio.map((p) => p.ticker);
  const marketStackMetrics = associateBy(
    await new CachingMarketStackScrapper(tickers).run(),
    (p) => p.ticker,
  );
  const productMetrics: ProductMetric[] = [];
  for (const product of portfolio) {
    const marketStackMetric = marketStackMetrics[product.ticker];
    productMetrics.push(await product.assembleMetrics(marketStackMetric, page));
  }
  return productMetrics;
}

export abstract class Product {
  protected constructor(public readonly ticker: string) {}
  abstract assembleMetrics(
    marketStackMetric: market_stack.ProductMetric,
    page: ManagedBrowserPage,
  ): Promise<ProductMetric>;
}

export class Stock extends Product {
  constructor(ticker: string) {
    super(ticker);
  }

  override async assembleMetrics(
    marketStackMetric: market_stack.ProductMetric,
    page: ManagedBrowserPage,
  ): Promise<ProductMetric> {
    const yahooMetric = await new CachingYahooFinanceStockScrapper(
      this.ticker,
      page,
    ).run();
    const oneMonthPriceChange =
      (yahooMetric.latestPrice - marketStackMetric.price1MonthAgo) /
      marketStackMetric.price1MonthAgo;
    return {
      ticker: this.ticker,
      name: marketStackMetric.name,
      long_term_total_return: yahooMetric.longTermTotalReturn,
      one_month_price_change: oneMonthPriceChange,
    };
  }
}

export class Etf extends Product {
  constructor(
    ticker: string,
    private isharesId: string,
    private isharesRegion: Region,
  ) {
    super(ticker);
  }

  override async assembleMetrics(
    marketStackMetric: market_stack.ProductMetric,
    page: ManagedBrowserPage,
  ): Promise<ProductMetric> {
    const yahooMetric = await new CachingYahooFinanceEtfScrapper(
      this.ticker,
      page,
    ).run();
    const isharesMetric = await new CachingIsharesScrapper(
      this.ticker,
      this.isharesId,
      this.isharesRegion,
      page,
    ).run();
    const oneMonthPriceChange =
      (yahooMetric.latestPrice - marketStackMetric.price1MonthAgo) /
      marketStackMetric.price1MonthAgo;
    return {
      ticker: this.ticker,
      name: marketStackMetric.name,
      long_term_total_return: isharesMetric.longTermTotalReturn,
      one_month_price_change: oneMonthPriceChange,
    };
  }
}

function getBrowserChannel(): string {
  const channelEnv = Deno.env.get("PLAYWRIGHT_BROWSER");
  return channelEnv === undefined ? "msedge" : channelEnv;
}
