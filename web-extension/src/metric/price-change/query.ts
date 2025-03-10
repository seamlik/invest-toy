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

  return parseFloat(changeInPercent) / 100;
}
