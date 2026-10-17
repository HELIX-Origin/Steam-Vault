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

export const platformUbisoftPlugin: UniversalPlugin = {
  id: "platform-ubisoft",
  name: "Ubisoft Connect Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Ubisoft Connect registry entries and configurations.",
  homepage: "https://ubisoftconnect.com",
  features: ["discovery", "launch", "uplay_protocol", "social", "achievements"],

  // Dynamic platform metadata
  icon: "🌀",
  platformId: "ubisoft",
  platformColor: "#06b6d4",
  defaultStoreUrl: "https://store.ubisoft.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["UbisoftConnect.exe", "upc.exe", "Uplay.exe"],
  wingetPackageId: "Ubisoft.Connect",
  downloadUrl: "https://ubisoftconnect.com/",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Ubisoft Connect Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'upc.exe') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "upc.exe (leave blank to autodetect)"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Ubisoft Connect detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "Ubisoft Connect registry scanner and uplay:// protocol runner active." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customUplayPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["upc.exe"], custom || null);
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
          message: `Ubisoft Connect successfully installed via winget into ${options?.targetDir || "default location"}.`
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
        message: "Opened official Ubisoft Connect download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Ubisoft Connect."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `uplay://launch/${game.appId}/0`
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
    return game.appId ? `uplay://install/${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `uplay://uninstall/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://store.ubisoft.com/game?pid=${game.appId}` : null;
  },

  getFriendsListUri(): string | null {
    return "uplay://friends";
  },

  getAchievementsUri(game): string | null {
    return game.appId ? `uplay://achievements/${game.appId}` : null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files (x86)\\Ubisoft\\Ubisoft Game Launcher\\games",
      "D:\\Ubisoft Games"
    ];
  }
};
