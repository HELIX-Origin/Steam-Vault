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

export const platformAmazonPlugin: UniversalPlugin = {
  id: "platform-amazon",
  name: "Amazon Games Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Amazon Games client installations and direct game binaries.",
  homepage: "https://gaming.amazon.com",
  features: ["discovery", "launch", "amazon_games_protocol", "nile_cli"],

  // Dynamic platform metadata
  icon: "📦",
  platformId: "amazon",
  platformColor: "#eab308",
  defaultStoreUrl: "https://gaming.amazon.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["Amazon Games.exe", "AmazonGames.exe", "nile.exe", "nile"],
  wingetPackageId: "Amazon.Games",
  downloadUrl: "https://gaming.amazon.com/amazon-games-app",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Amazon Games / Nile Binary or Path",
          type: "text",
          description: "Optional binary name (e.g. 'Amazon Games.exe' or 'nile') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "Amazon Games.exe (leave blank to autodetect)"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Amazon Games detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "Amazon Games scanner active." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customClientPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["Amazon Games.exe"], custom || null);
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
          message: `Amazon Games successfully installed via winget into ${options?.targetDir || "default location"}.`
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
        message: "Opened official Amazon Games download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Amazon Games."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `amazon-games://play/${game.appId}`
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
    return game.appId ? `amazon-games://play/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://gaming.amazon.com/home` : null;
  },

  getDefaultKnownFolders(): string[] {
    return [
      "%LOCALAPPDATA%\\Amazon Games\\Library",
      "C:\\Amazon Games\\Library",
      "D:\\Amazon Games"
    ];
  }
};
