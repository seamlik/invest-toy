import { visit } from "./yahoo-finance.js";

chrome.action.onClicked.addListener((tab) => {
  visit();
});
