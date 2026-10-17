import type { UniversalPlugin } from "../framework";

export const storageAnalyzerPlugin: UniversalPlugin = {
  id: "utility-storage",
  name: "Storage & Drive Analyzer",
  version: "1.0.0",
  pluginType: "utility",
  author: "HELIX-Origin",
  description: "Calculates disk footprint across all platform directories and library drives.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["disk_analysis", "drive_reporting", "non_blocking_worker"],
  getSettingsSchema: () => ({
    requiresApiKey: false,
    fields: [
      {
        key: "backgroundSync",
        label: "Background Async Inspection",
        type: "boolean",
        default: true,
        description: "Run drive disk usage calculations on a background worker thread to prevent UI freezing."
      }
    ]
  })
};
