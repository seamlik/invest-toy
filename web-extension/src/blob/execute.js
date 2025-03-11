// This file has to be written in JavaScript
// because of a mistake in the declarations from `chrome-types`.
// The `func` parameter below is declared as () => void,
// which means it cannot accept any arguments.

import { createJsonBlobUrl } from "./url";

export async function createJsonBlobUrlInTab(content, tabId) {
  const [result] = await chrome.scripting.executeScript({
    target: { tabId: tabId },
    func: createJsonBlobUrl,
    args: [content],
  });
  return result.result;
}
