import { generateReport } from "./yahoo-finance";

chrome.action.onClicked.addListener((tab) => {
  console.info(`Service Worker clicked on tab ${tab.id?.toString() ?? "null"}`);
  generateReport().catch((e: unknown) => {
    console.error(e);
  });
});
