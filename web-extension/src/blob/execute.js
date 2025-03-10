// See `../price-change/extract.js` for rationale of writing this in JavaScript

import { navigateToBlob } from "./navigate.js";

export async function navigateToBlobInTab(tabId, blob) {
  await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: navigateToBlob,
    args: [blob],
  });
}
