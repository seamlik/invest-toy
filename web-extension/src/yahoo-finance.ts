export function visit() {
  chrome.tabs.create({
    url: "https://finance.yahoo.com",
  });
}
