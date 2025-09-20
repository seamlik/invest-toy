const apiKeyEnv = "MARKET_STACK_API_KEY";
const apiKey = Deno.env.get(apiKeyEnv)?.trim();

export async function scrapProducts(
  tickers: string[],
): Promise<ProductMetric[]> {
  if (apiKey === undefined || apiKey.length === 0) {
    throw new Error(`Missing environment variable ${apiKeyEnv}`);
  }

  const prices1MonthAgo = await scrapPrices1MonthAgo(tickers);
  const products = await scrapProductsWithLatestPrices(tickers);
  return products.map((product) => {
    const price1MonthAgo = prices1MonthAgo.get(product.symbol);
    if (price1MonthAgo == undefined) {
      throw new Error(
        `Product ${product.symbol} exists in the latest data but not 1 month ago`,
      );
    }
    return buildProductMetric(product, price1MonthAgo);
  });
}

async function scrapPrices1MonthAgo(
  tickers: string[],
): Promise<Map<string, number>> {
  const date1MonthAgo = Temporal.Now.plainDateISO().subtract({ days: 30 });
  const tickersJoined = tickers.join(",");
  const url = `https://api.marketstack.com/v2/eod/${date1MonthAgo}?access_key=${apiKey}&symbols=${tickersJoined}`;
  const response = (await (await fetch(url)).json()) as MarketStackResponse;
  return new Map(
    response.data.map((product) => [product.symbol, product.close]),
  );
}

async function scrapProductsWithLatestPrices(
  tickers: string[],
): Promise<MarketStackProduct[]> {
  const tickersJoined = tickers.join(",");
  const url = `https://api.marketstack.com/v2/eod/latest?access_key=${apiKey}&symbols=${tickersJoined}`;
  return ((await (await fetch(url)).json()) as MarketStackResponse).data;
}

function buildProductMetric(
  product: MarketStackProduct,
  price1MonthAgo: number,
): ProductMetric {
  return {
    ticker: product.symbol,
    name: product.name,
    oneMonthPriceChange: (product.close - price1MonthAgo) / price1MonthAgo,
  };
}

export interface ProductMetric {
  ticker: string;
  name: string;
  oneMonthPriceChange: number;
}

interface MarketStackResponse {
  data: MarketStackProduct[];
}

interface MarketStackProduct {
  close: number;
  name: string;
  symbol: string;
}
