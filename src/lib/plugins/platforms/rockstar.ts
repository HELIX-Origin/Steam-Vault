import type {
  UniversalPlugin,
  TestConnectionResult,
  PlatformLaunchSpec,
  LauncherDetectionResult,
  LauncherInstallResult,
  LauncherInstallOptions,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";

export const platformRockstarPlugin: UniversalPlugin = {
  id: "platform-rockstar",
  name: "Rockstar Games Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Rockstar Games Launcher installations and rockstar:// URI scheme.",
  homepage: "https://www.rockstargames.com",
  features: ["discovery", "launch", "rockstar_protocol"],

  // Dynamic platform metadata
  icon: "★",
  platformId: "rockstar",
  platformColor: "#fbbf24",
  defaultStoreUrl: "https://store.rockstargames.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["Launcher.exe", "RockstarService.exe", "PlayGTAV.exe"],
  wingetPackageId: "RockstarGames.RockstarGamesLauncher",
  downloadUrl: "https://socialclub.rockstargames.com/rockstar-games-launcher",
  supportsSilentInstall: false,
  supportsCustomInstallPath: false,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Rockstar Games Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'Launcher.exe') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "Launcher.exe (leave blank to autodetect)"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Rockstar Games detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "Rockstar Games scanner and protocol runner operational." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customLauncherPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["Launcher.exe"], custom || null);
    if (detected) {
      return {
        installed: true,
        executablePath: detected,
        detectedBinary: detected,
        method: custom ? "custom_path" : "path_binary",
        downloadUrl: this.downloadUrl,
        supportsSilentInstall: false,
        wingetPackageId: this.wingetPackageId,
      };
    }

    return {
      installed: false,
      downloadUrl: this.downloadUrl,
      supportsSilentInstall: false,
      wingetPackageId: this.wingetPackageId,
      method: "not_installed",
    };
  },

  async installLauncher(_options?: LauncherInstallOptions, _config?: Record<string, any>): Promise<LauncherInstallResult> {
    if (this.downloadUrl) {
      await api.openExternalUrl(this.downloadUrl);
      return {
        success: true,
        message: "Opened official Rockstar Games Launcher download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL available for Rockstar Games Launcher."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `rockstar://launch/${game.appId}`
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
    return game.appId ? `rockstar://launch/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://store.rockstargames.com/game/buy-${game.appId}` : null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files\\Rockstar Games",
      "D:\\Rockstar Games"
    ];
  }
};
