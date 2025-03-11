export function extractTicker(): string | null {
  const element = document.querySelector(
    'section[data-testid="quote-hdr"] div.hdr h1',
  );
  return element instanceof HTMLHeadingElement ? element.textContent : null;
}
