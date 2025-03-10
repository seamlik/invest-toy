// This file has to be written in JavaScript
// because of a mistake in the declarations from `chrome-types`.
// The `func` parameter below is declared as () => void,
// which is incompatible with `queryPriceChange`.

import { queryPriceChange } from "./query.js";

export async function extractPriceChange(tabId, changePeriod) {
  const [result] = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: queryPriceChange,
    args: [changePeriod],
  });
  return result.result;
}
