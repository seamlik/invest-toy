import { BrowserContext, devices, Page } from "playwright";
import { chromium, Browser } from "playwright";

export class ManagedBrowserPage implements AsyncDisposable {
  static async create(channel: string): Promise<ManagedBrowserPage> {
    const browser = await chromium.launch({ channel: channel });
    try {
      const context = await browser.newContext({
        ...devices["Desktop Chrome"],
      });
      try {
        const page = await context.newPage();
        return new ManagedBrowserPage(browser, context, page);
      } catch (e) {
        await context.close();
        throw e;
      }
    } catch (e) {
      await browser.close();
      throw e;
    }
  }

  private constructor(
    private browser: Browser,
    private context: BrowserContext,
    public page: Page,
  ) {}

  async [Symbol.asyncDispose](): Promise<void> {
    await this.page.close();
    await this.context.close();
    await this.browser.close();
  }

  async goto(url: string): Promise<void> {
    console.info(`Loading ${url}`);
    await this.page.goto(url, { waitUntil: "domcontentloaded" });
  }
}
