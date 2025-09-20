import { Page } from "playwright";

export async function navigate(page: Page, url: string): Promise<void> {
  console.info(`Loading ${url}`);
  await page.goto(url, { waitUntil: "domcontentloaded" });
}
