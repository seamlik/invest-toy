import * as csv from "@std/csv";
import { Region } from "./scrapper/ishares.ts";

export function load(csvText: string): Product[] {
  const csvValues = csv.parse(csvText, {
    columns: ["ticker", "type", "isharedId", "isharesRegion"],
    skipFirstRow: true,
  });
  return csvValues.map((row) => {
    return {
      ticker: row.ticker,
      type: parseProductType(row.type),
      isharesId: row.isharedId,
      isharesRegion: Region.parse(row.isharesRegion.trim()),
    };
  });
}

export interface Product {
  ticker: string;
  type: ProductType;
  isharesId: string;
  isharesRegion?: Region;
}

export enum ProductType {
  Stock = "Stock",
  Etf = "Exchange-Traded Fund",
}

function parseProductType(source: string): ProductType {
  switch (source) {
    case ProductType.Stock:
      return ProductType.Stock;
    case ProductType.Etf:
      return ProductType.Etf;
    default:
      throw new Error(`Unknown product type ${source}`);
  }
}
