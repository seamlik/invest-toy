import { createJsonBlobUrlInTab } from "./execute";

export async function downloadBlobFromTab(
  content: unknown,
  tabId: number,
  fileName: string,
) {
  const url = await createJsonBlobUrlInTab(content, tabId);
  await chrome.downloads.download({
    filename: fileName,
    url: url,
    saveAs: true,
  });
}
