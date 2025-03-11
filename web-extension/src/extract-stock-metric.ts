import { StockMetric } from "../../json-schema/typescript";
import { extractDividendYield } from "./metric/dividend-yield";
import { extractPriceChange } from "./metric/price-change";
import { extractTicker } from "./metric/ticker";

const metric = extractStockMetric();
chrome.runtime.sendMessage(metric).catch(console.error);

function extractStockMetric(): StockMetric | null {
  const ticker = extractTicker();
  if (ticker === null) {
    return null;
  }

  return {
    ticker: ticker,
    dividend_yield: extractDividendYield() ?? undefined,
    price_change_in_one_month: extractPriceChange("1m") ?? undefined,
    price_change_in_five_years: extractPriceChange("5y") ?? undefined,
  };
}
