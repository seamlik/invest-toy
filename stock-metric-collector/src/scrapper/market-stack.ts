const apiKeyEnv = "MARKET_STACK_API_KEY";
const apiKey = Deno.env.get(apiKeyEnv)?.trim();

export async function scrapProducts(
  tickers: string[],
): Promise<ProductMetric[]> {
  if (apiKey === undefined || apiKey.length === 0) {
    throw new Error(`Missing environment variable ${apiKeyEnv}`);
  }

  const date1MonthAgo = Temporal.Now.plainDateISO().subtract({ days: 30 });
  const tickersJoined = tickers.join(",");
  const url = `https://api.marketstack.com/v2/eod/${date1MonthAgo}?access_key=${apiKey}&symbols=${tickersJoined}`;
  console.info(`Fetching ${url}`);
  const response = (await (await fetch(url)).json()) as MarketStackResponse;
  return response.data.map((product) => {
    return {
      ticker: product.symbol,
      name: product.name,
      price1MonthAgo: product.close,
    };
  });
}

export interface ProductMetric {
  ticker: string;
  name: string;
  price1MonthAgo: number;
}

interface MarketStackResponse {
  data: MarketStackProduct[];
}

interface MarketStackProduct {
  close: number;
  name: string;
  symbol: string;
}
