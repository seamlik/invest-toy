export function createJsonBlobUrl(content: unknown): string {
  const json = JSON.stringify(content);
  const blob = new Blob([json], { type: "application/json" });
  return URL.createObjectURL(blob);
}
