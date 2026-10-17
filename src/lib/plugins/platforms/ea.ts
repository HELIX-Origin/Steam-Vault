import type {
  UniversalPlugin,
  TestConnectionResult,
  PlatformLaunchSpec,
  LauncherDetectionResult,
  LauncherInstallResult,
  LauncherInstallOptions,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";

export const platformEaPlugin: UniversalPlugin = {
  id: "platform-ea",
  name: "EA Desktop Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Electronic Arts (EA Desktop & Origin) local installations.",
  homepage: "https://www.ea.com",
  features: ["discovery", "launch", "origin_protocol", "social", "achievements"],

  // Dynamic platform metadata
  icon: "●",
  platformId: "ea",
  platformColor: "#f97316",
  defaultStoreUrl: "https://www.ea.com/games",

  // Provider launcher capabilities
  launcherBinaryNames: ["EADesktop.exe", "EALauncher.exe", "Origin.exe"],
  wingetPackageId: "ElectronicArts.EADesktop",
  downloadUrl: "https://www.ea.com/ea-app",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom EA Desktop Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'EALauncher.exe') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "EALauncher.exe (leave blank to autodetect)"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `EA Desktop detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "EA Desktop registry and launch protocol runner active." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customEaPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["EADesktop.exe"], custom || null);
    if (detected) {
      return {
        installed: true,
        executablePath: detected,
        detectedBinary: detected,
        method: custom ? "custom_path" : "path_binary",
        downloadUrl: this.downloadUrl,
        supportsSilentInstall: true,
        wingetPackageId: this.wingetPackageId,
      };
    }

    return {
      installed: false,
      downloadUrl: this.downloadUrl,
      supportsSilentInstall: true,
      wingetPackageId: this.wingetPackageId,
      method: "not_installed",
    };
  },

  async installLauncher(options?: LauncherInstallOptions, _config?: Record<string, any>): Promise<LauncherInstallResult> {
    const wingetReady = await api.isWingetAvailable();
    if (wingetReady && this.wingetPackageId) {
      try {
        await api.installWingetPackage(this.wingetPackageId, options?.targetDir || null);
        return {
          success: true,
          message: `EA Desktop successfully installed via winget into ${options?.targetDir || "default location"}.`
        };
      } catch (err: any) {
        if (this.downloadUrl) {
          await api.openExternalUrl(this.downloadUrl);
        }
        return {
          success: false,
          message: `WinGet installation failed: ${err?.message || err}. Opened official download page.`
        };
      }
    }

    if (this.downloadUrl) {
      await api.openExternalUrl(this.downloadUrl);
      return {
        success: true,
        message: "Opened official EA App download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for EA Desktop."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `origin2://game/launch?offerIds=${game.appId}`
      };
    }
    if (game.exe) {
      return {
        executablePath: game.exe,
        args: game.launchArgs ? [game.launchArgs] : []
      };
    }
    return null;
  },

  getInstallUri(game): string | null {
    return game.appId ? `origin2://game/launch?offerIds=${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `origin2://game/launch?offerIds=${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://www.ea.com/games/library/${game.appId}` : null;
  },

  getFriendsListUri(): string | null {
    return "origin2://friends/show";
  },

  getAchievementsUri(game): string | null {
    return game.appId ? `origin2://game/achievements?offerIds=${game.appId}` : null;
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;
    return {
      name: term,
      shortDescription: `Electronic Arts title: ${term}.`,
      detailedDescription: `Electronic Arts game available through EA Desktop & Origin.`,
      publishers: ["Electronic Arts"],
      developers: ["Electronic Arts"],
      categories: ["EA Desktop", "Electronic Arts"]
    };
  },

  async queryArtwork(_query): Promise<PluginArtworkResult | null> {
    return null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files\\EA Games",
      "C:\\Program Files (x86)\\Origin Games",
      "D:\\EA Games"
    ];
  }
};
