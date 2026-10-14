// User-agent based OS detection, used to pick native-looking window controls.

export type OsPlatform = "windows" | "macos" | "linux" | "other";

export function getPlatform(): OsPlatform {
  if (typeof navigator === "undefined") return "windows";
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac") || ua.includes("darwin")) return "macos";
  if (ua.includes("linux")) return "linux";
  if (ua.includes("win")) return "windows";
  return "other";
}
