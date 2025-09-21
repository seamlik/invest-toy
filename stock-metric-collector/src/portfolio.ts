import * as csv from "@std/csv";
import { Region } from "./scrapper/ishares.ts";
import { Etf, Product, Stock } from "./collector.ts";

export function load(csvText: string): Product[] {
  const csvValues = csv.parse(csvText, {
    columns: ["ticker", "type", "isharesId", "isharesRegion"],
    skipFirstRow: true,
  });
  return csvValues.map((row) => {
    switch (row.type) {
      case "Stock":
        return new Stock(row.ticker);
      case "Exchange-Traded Fund":
        return new Etf(
          row.ticker,
          row.isharesId,
          Region.parse(row.isharesRegion.trim()),
        );
      default:
        throw new Error(`Unknown product type ${row.type}`);
    }
  });
}
