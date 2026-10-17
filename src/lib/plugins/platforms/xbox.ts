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

export const platformXboxPlugin: UniversalPlugin = {
  id: "platform-xbox",
  name: "Xbox & Microsoft Store Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Xbox Game Pass, Microsoft Store packages, and WindowsApps folders.",
  homepage: "https://www.xbox.com",
  features: ["discovery", "launch", "game_pass", "uwp_packages", "social", "achievements"],

  // Dynamic platform metadata
  icon: "⨂",
  platformId: "xbox",
  platformColor: "#22c55e",
  defaultStoreUrl: "https://www.xbox.com/games/store",

  // Provider launcher capabilities
  launcherBinaryNames: ["XboxApp.exe", "XboxPcApp.exe"],
  wingetPackageId: "Microsoft.GamingApp",
  downloadUrl: "https://www.xbox.com/apps/xbox-app-for-pc",
  supportsSilentInstall: true,
  supportsCustomInstallPath: false,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Xbox App Binary / Path",
          type: "text",
          description: "Optional binary name or path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "XboxApp.exe (leave blank to autodetect)"
        },
        {
          key: "scanGamePass",
          label: "Scan Xbox Game Pass Installations",
          type: "boolean",
          default: true,
          description: "Search all local drive partitions for XboxGames and WindowsApps packages."
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Xbox App detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "WindowsApps package inspector and shell:AppsFolder launcher operational." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["XboxApp.exe"], custom || null);
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

    // On Windows 10/11 Xbox is typically registered as a system UWP app
    return {
      installed: true,
      executablePath: "Microsoft.GamingApp",
      detectedBinary: "XboxApp.exe",
      method: "system_store",
      downloadUrl: this.downloadUrl,
      supportsSilentInstall: true,
      wingetPackageId: this.wingetPackageId,
    };
  },

  async installLauncher(options?: LauncherInstallOptions, _config?: Record<string, any>): Promise<LauncherInstallResult> {
    const wingetReady = await api.isWingetAvailable();
    if (wingetReady && this.wingetPackageId) {
      try {
        await api.installWingetPackage(this.wingetPackageId, options?.targetDir || null);
        return {
          success: true,
          message: "Xbox App successfully installed via winget."
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
        message: "Opened official Xbox App download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Xbox App."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.appId) {
      return {
        protocolUri: `shell:AppsFolder\\${game.appId}!App`
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
    return game.appId ? `ms-windows-store://pdp/?ProductId=${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://www.xbox.com/games/store/-/${game.appId}` : null;
  },

  getFriendsListUri(): string | null {
    return "ms-xbox-overlay:friends";
  },

  getAchievementsUri(game): string | null {
    return "ms-xbox-overlay:achievements";
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\XboxGames",
      "D:\\XboxGames",
      "C:\\Program Files\\WindowsApps"
    ];
  }
};
