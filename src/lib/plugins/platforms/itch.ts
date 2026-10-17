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

export const platformItchPlugin: UniversalPlugin = {
  id: "platform-itch",
  name: "itch.io Platform Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games installed via the itch.io desktop client and standalone receipt manifests.",
  homepage: "https://itch.io",
  features: ["discovery", "launch", "indie_installations", "butler_cli"],

  // Dynamic platform metadata
  icon: "⌗",
  platformId: "itch",
  platformColor: "#ef4444",
  defaultStoreUrl: "https://itch.io",

  // Provider launcher capabilities
  launcherBinaryNames: ["itch.exe", "itch", "butler.exe", "butler"],
  wingetPackageId: "itchio.itch",
  downloadUrl: "https://itch.io/app",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom itch.io / Butler Binary or Path",
          type: "text",
          description: "Optional binary name (e.g. 'itch.exe' or 'butler') or custom path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "itch.exe (leave blank to autodetect)"
        },
        {
          key: "customItchRoot",
          label: "Custom itch.io App Data Folder",
          type: "text",
          description: "Optional custom path to the itch.io app database directory.",
          placeholder: "%APPDATA%\\itch"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `itch.io detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return { success: true, message: "itch.io local directory and butler receipt scanner active." };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customItchRoot?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["itch.exe", "butler"], custom || null);
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
          message: `itch.io successfully installed via winget into ${options?.targetDir || "default location"}.`
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
        message: "Opened official itch.io download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for itch.io."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game): PlatformLaunchSpec | null {
    if (game.exe) {
      return {
        executablePath: game.exe,
        args: game.launchArgs ? [game.launchArgs] : []
      };
    }
    if (game.appId) {
      return {
        protocolUri: `itch://games/${game.appId}`
      };
    }
    return null;
  },

  getInstallUri(game): string | null {
    return game.appId ? `itch://games/${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `itch://games/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://itch.io/t/${game.appId}` : null;
  },

  async queryMetadata(query, _config): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch(`https://itch.io/search?q=${encodeURIComponent(term)}&format=json`);
      if (!res.ok) return null;
      const data = await res.json();
      const games: any[] = data?.games || [];
      const g = games[0];
      if (!g) return null;

      const dev = g.user?.name || g.user?.username;

      return {
        name: g.title || term,
        shortDescription: g.short_text,
        detailedDescription: g.short_text,
        developers: dev ? [dev] : undefined,
        publishers: dev ? [dev] : undefined,
        capsuleImage: g.cover_url,
        heroImage: g.cover_url,
        headerImage: g.cover_url,
        websiteUrl: g.url
      };
    } catch {
      return null;
    }
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    const meta = await this.queryMetadata!(query, config);
    if (!meta) return null;
    return {
      capsuleImage: meta.capsuleImage,
      heroImage: meta.heroImage,
      headerImage: meta.headerImage
    };
  },

  getDefaultKnownFolders(): string[] {
    return [
      "%APPDATA%\\itch\\apps",
      "%LOCALAPPDATA%\\itch\\apps"
    ];
  }
};
