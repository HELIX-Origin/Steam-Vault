import type {
  UniversalPlugin,
  TestConnectionResult,
  PlatformLaunchSpec,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const platformStandalonePlugin: UniversalPlugin = {
  id: "platform-standalone",
  name: "Standalone & Local Executables",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Enables importing and launching arbitrary local games, DRM-free titles, emulators, and custom executables.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["discovery", "launch", "custom_executables", "emulators", "drm_free", "manual_import"],

  // Dynamic platform metadata
  icon: "⚙",
  platformId: "standalone",
  platformLabel: "Standalone & Local",
  platformColor: "#94a3b8",
  defaultStoreUrl: "",

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "defaultWorkingDir",
          label: "Inherit Game Directory as Working Directory",
          type: "boolean",
          default: true,
          description: "Sets the current working directory to the game executable's folder when launching."
        }
      ]
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    return { success: true, message: "Standalone process execution engine ready." };
  },

  async detectLauncher() {
    return {
      installed: true,
      method: "standalone" as const,
      message: "Direct standalone execution requires no third-party desktop client."
    };
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.exe) {
      return {
        executablePath: game.exe,
        args: game.launchArgs ? [game.launchArgs] : []
      };
    }
    return null;
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const rawTitle = query.name || "";
    const cleanTitle = rawTitle.replace(/\.[a-zA-Z0-9]+$/, "").replace(/[-_]/g, " ").trim();
    return {
      name: cleanTitle || rawTitle,
      shortDescription: "Local standalone executable game.",
      detailedDescription: "Local standalone executable game imported into Universal Game Launcher.",
      categories: ["Standalone", "Local"],
      extra: {
        source: "Standalone"
      }
    };
  },

  async queryArtwork(): Promise<PluginArtworkResult | null> {
    return null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Games",
      "D:\\Games"
    ];
  }
};
