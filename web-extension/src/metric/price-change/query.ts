export function queryPriceChange(changeType: string): number | null {
  const button = document.querySelector(`button#tab-${changeType}`);
  if (!(button instanceof HTMLButtonElement)) {
    return null;
  }

  button.click();

  const tooltip = button.querySelector("div.tooltip h3");
  if (!(tooltip instanceof HTMLHeadingElement)) {
    return null;
  }

  const changeInPercent = tooltip.textContent;
  if (changeInPercent === null) {
    return null;
  }

  // Here the logic of parsing percentages is repeated.
  // This is because we cannot call another function.
  // This is because any function injected into a tab must be self-contained.
  const stripped = changeInPercent
    .replaceAll(",", "")
    .replaceAll("%", "")
    .trim();
  return parseFloat(stripped) / 100;
}
