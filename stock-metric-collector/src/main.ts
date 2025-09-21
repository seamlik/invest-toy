import * as io from "@std/io";
import * as portfolio from "./portfolio.ts";
import { ManagedBrowserPage } from "./playwright.ts";

async function main(): Promise<void> {
  const channel = getBrowserChannel();
  const portfolio = await loadPortfolioFromStdIn();
  await using page = await ManagedBrowserPage.create(channel);
}

function getBrowserChannel(): string {
  const channelEnv = Deno.env.get("PLAYWRIGHT_BROWSER");
  return channelEnv === undefined ? "msedge" : channelEnv;
}

async function loadPortfolioFromStdIn(): Promise<portfolio.Product[]> {
  const csvTextBuffer = await io.readAll(Deno.stdin);
  const csvText = new TextDecoder("utf-8").decode(csvTextBuffer);
  return portfolio.load(csvText);
}

await main();
