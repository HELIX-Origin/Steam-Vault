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

export const platformBattlenetPlugin: UniversalPlugin = {
  id: "platform-battlenet",
  name: "Battle.net Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Blizzard Battle.net installations and battle.net:// URI protocol.",
  homepage: "https://battle.net",
  features: ["discovery", "launch", "battlenet_protocol", "social", "achievements"],

  // Dynamic platform metadata
  icon: "❄",
  platformId: "battlenet",
  platformColor: "#0284c7",
  defaultStoreUrl: "https://battle.net",

  // Provider launcher capabilities
  launcherBinaryNames: ["Battle.net.exe", "Battle.net Launcher.exe"],
  wingetPackageId: "Blizzard.BattleNet",
  downloadUrl: "https://www.battle.net/download/getInstaller",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Battle.net Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'Battle.net.exe') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "Battle.net.exe (leave blank to autodetect)"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Battle.net detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "Battle.net directory scanner and URI protocol active." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customClientPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["Battle.net.exe"], custom || null);
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
          message: `Battle.net successfully installed via winget into ${options?.targetDir || "default location"}.`
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
        message: "Opened official Battle.net download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Battle.net."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `battlenet://${game.appId}`
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
    return game.appId ? `battlenet://${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `battlenet://${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://shop.battle.net/family/${game.appId}` : null;
  },

  getFriendsListUri(): string | null {
    return "battlenet://friends";
  },

  getAchievementsUri(game): string | null {
    return game.appId ? `battlenet://achievements/${game.appId}` : null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files (x86)\\Battle.net",
      "C:\\Program Files\\Battle.net",
      "D:\\Battle.net Games"
    ];
  }
};
