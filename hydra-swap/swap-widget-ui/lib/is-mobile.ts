import { detect } from "detect-browser";

export function isMobile() {
  const browser = detect();
  return (
    browser?.os === "Android OS" ||
    browser?.os === "iOS" ||
    browser?.os === "Windows Mobile"
  );
}
