export function parsePercentage(source: string): number {
  const stripped = source.replaceAll(",", "").replaceAll("%", "").trim();
  return parseFloat(stripped) / 100;
}
