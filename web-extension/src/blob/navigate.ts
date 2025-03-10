export function navigateToBlob(source: unknown) {
  const json = JSON.stringify(source);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  window.location.href = url;
}
