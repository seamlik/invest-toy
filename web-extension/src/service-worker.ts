import { generateReport } from "./yahoo-finance.js";

chrome.action.onClicked.addListener((tab) => {
  generateReport();
});
