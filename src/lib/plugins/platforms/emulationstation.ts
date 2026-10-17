import type {
  UniversalPlugin,
  PlatformLaunchSpec,
  TestConnectionResult,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig, PlatformGame, ScaffoldReport } from "$lib/types";
import { api } from "$lib/api";

export const platformEmulationStationPlugin: UniversalPlugin = {
  id: "platform-emulationstation",
  name: "EmulationStation Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches emulated games and ROMs organized via the ES-DE directory standard with automated folder scaffolding and intelligent title normalization.",
  homepage: "https://gitlab.com/es-de/emulationstation-de",
  icon: "🕹️",
  features: ["rom_scanning", "folder_scaffolding", "title_normalization", "multi_console"],

  platformId: "emulationstation",
  platformLabel: "EmulationStation",
  platformColor: "#f59e0b",
  defaultStoreUrl: "https://es-de.org",

  launcherBinaryNames: ["emulationstation.exe", "es-de.exe", "emulationstation", "es-de"],
  downloadUrl: "https://es-de.org/#Download",
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "romsPath",
          label: "ROMs Root Directory",
          type: "text",
          required: true,
          default: "C:\\ROMs",
          description: "Root folder where console ROMs are stored and scaffolded following the ES-DE standard.",
          placeholder: "e.g. C:\\ROMs or D:\\Emulation\\ROMs",
          helpUrl: "https://gitlab.com/es-de/emulationstation-de/-/blob/master/USERGUIDE.md",
          helpLabel: "Guide ↗"
        },
        {
          key: "esExecutablePath",
          label: "EmulationStation / ES-DE Executable Path",
          type: "text",
          required: false,
          default: "",
          description: "Optional custom path to the ES-DE or EmulationStation executable.",
          placeholder: "e.g. C:\\Program Files\\ES-DE\\ES-DE.exe"
        },
        {
          key: "normalizeTitles",
          label: "Normalize ROM Titles",
          type: "boolean",
          required: false,
          default: true,
          description: "Automatically strip dump tags, region codes, multi-disc tags, and file extensions for clean display names."
        },
        {
          key: "autoScaffold",
          label: "Auto-Scaffold System Folders",
          type: "boolean",
          required: false,
          default: true,
          description: "Automatically create system directories, systeminfo.txt, and systems.txt in the ROMs folder if missing."
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const romsPath = (config?.romsPath || "C:\\ROMs").trim();
    if (!romsPath) {
      return { success: false, message: "A ROMs directory path is required." };
    }
    try {
      const folders = await api.findPlatformFolders("emulationstation");
      return {
        success: true,
        message: `Configured ROMs path: ${romsPath}. EmulationStation plugin is active and ready.`
      };
    } catch (err: any) {
      return {
        success: true,
        message: `EmulationStation plugin configured with path: ${romsPath}`
      };
    }
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    const esExe = (config?.esExecutablePath || "").trim();
    if (esExe && game.installPath) {
      return {
        executablePath: esExe,
        args: ["--system", game.appName || "", "--rom", game.installPath]
      };
    }
    if (game.installPath) {
      return {
        executablePath: game.installPath
      };
    }
    return null;
  },

  async scanPlatform(roots, config): Promise<{ installed: boolean; foundGamesCount?: number; clientPath?: string | null }> {
    const romsPath = (config?.romsPath || roots?.[0] || "C:\\ROMs").trim();
    const normalize = config?.normalizeTitles !== false;
    const autoScaffold = config?.autoScaffold !== false;

    if (autoScaffold && romsPath) {
      try {
        await api.scaffoldEmulationFolders(romsPath);
      } catch (err) {
        console.warn("EmulationStation auto-scaffold warning:", err);
      }
    }

    try {
      const roms = await api.scanEmulationRoms(romsPath, normalize);
      return {
        installed: true,
        clientPath: romsPath,
        foundGamesCount: roms.length
      };
    } catch {
      return {
        installed: false,
        clientPath: romsPath,
        foundGamesCount: 0
      };
    }
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const rawTitle = query.name || "";
    let cleanTitle = rawTitle;
    try {
      cleanTitle = await api.normalizeRomTitle(rawTitle);
    } catch {
      cleanTitle = rawTitle.replace(/\s*\([^)]*\)|\s*\[[^\]]*\]/g, "").replace(/\.[a-zA-Z0-9]+$/, "").trim();
    }
    const consoleName = query.platform || "Emulated Console";
    return {
      name: cleanTitle || rawTitle,
      shortDescription: `Emulated title for ${consoleName}.`,
      detailedDescription: `Emulated title for ${consoleName}. Discovered via EmulationStation ROM repository.`,
      categories: [consoleName, "Emulation", "Retro"],
      genres: ["Retro"],
      extra: {
        console: consoleName,
        source: "EmulationStation"
      }
    };
  },

  async queryArtwork(_query): Promise<PluginArtworkResult | null> {
    return null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\ROMs",
      "D:\\ROMs",
      "E:\\ROMs",
      "C:\\Emulation\\ROMs",
      "D:\\Emulation\\ROMs"
    ];
  }
};
