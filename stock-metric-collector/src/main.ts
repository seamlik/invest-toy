import * as io from "@std/io";
import * as portfolio from "./portfolio.ts";
import { collectProductMetrics, Product } from "./collector.ts";
import { resolve } from "@std/path/resolve";
import { outputDirectoryPath } from "./scrapper/caching.ts";

async function main() {
  const portfolio = await loadPortfolioFromStdIn();
  const metrics = await collectProductMetrics(portfolio);
  const outputFilePath = resolve(outputDirectoryPath, "metrics.json");
  await Deno.writeTextFile(outputFilePath, JSON.stringify(metrics));
  console.info(`Metrics successfully wrote to ${outputFilePath}`);
}

async function loadPortfolioFromStdIn(): Promise<Product[]> {
  const csvTextBuffer = await io.readAll(Deno.stdin);
  const csvText = new TextDecoder("utf-8").decode(csvTextBuffer);
  return portfolio.load(csvText);
}

await main();
