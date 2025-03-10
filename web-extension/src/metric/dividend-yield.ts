queryDividendYield();

function queryDividendYield(): number | null {
  const items = document.querySelectorAll(
    'div[data-testid="quote-statistics"] > ul > li',
  );
  for (const element of items) {
    const dividendYield = extractDividendYield(element);
    if (dividendYield !== null) {
      return dividendYield;
    }
  }
  return null;
}

function extractDividendYield(element: Element): number | null {
  const spans = [
    ...element.querySelectorAll(":scope > span"),
  ] as HTMLSpanElement[];
  if (spans.length !== 2) {
    throw new Error(`Data cell has more than 2 spans: ${element.getHTML()}`);
  }

  const rawData = spans[1].textContent;
  if (rawData === null || rawData === "--") {
    return null;
  }

  switch (spans[0].textContent) {
    case "Forward Dividend & Yield":
      return convertForwardDividendYieldToNumber(rawData);
    case "Yield":
      return parsePercentage(rawData);
    default:
      return null;
  }
}

function convertForwardDividendYieldToNumber(source: string): number {
  const percentage = source.replaceAll(")", "").split("(")[1].trim();
  return parsePercentage(percentage);
}

function parsePercentage(source: string): number {
  const stripped = source.replaceAll(",", "").replaceAll("%", "").trim();
  return parseFloat(stripped) / 100;
}
