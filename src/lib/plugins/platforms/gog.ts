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

export const platformGogPlugin: UniversalPlugin = {
  id: "platform-gog",
  name: "GOG Galaxy Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from GOG Galaxy registry entries, local index databases, and DRM-free installations.",
  homepage: "https://www.gog.com/galaxy",
  features: ["discovery", "launch", "drm_free", "galaxy_client", "social", "achievements"],

  // Dynamic platform metadata
  icon: "◈",
  platformId: "gog",
  platformColor: "#a855f7",
  defaultStoreUrl: "https://www.gog.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["GalaxyClient.exe", "GalaxyCommunication.exe"],
  wingetPackageId: "GOG.Galaxy",
  downloadUrl: "https://www.gog.com/galaxy",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom GOG Galaxy Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'GalaxyClient.exe') or absolute executable path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "GalaxyClient.exe (leave blank to autodetect)"
        },
        {
          key: "launchViaGalaxy",
          label: "Launch via GOG Galaxy Client",
          type: "boolean",
          default: false,
          description: "When disabled, games launch DRM-free directly from their game executable without opening Galaxy."
        },
        {
          key: "customGalaxyPath",
          label: "Custom GalaxyClient.exe Path",
          type: "text",
          description: "Optional custom path to the GOG Galaxy client executable.",
          placeholder: "C:\\Program Files (x86)\\GOG Galaxy\\GalaxyClient.exe"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `GOG Galaxy detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
      };
    }
    return {
      success: true,
      message: "GOG DRM-free direct executable runner active (Galaxy client not detected on PATH)."
    };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customGalaxyPath?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["GalaxyClient.exe"], custom || null);
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
          message: `GOG Galaxy successfully installed via winget into ${options?.targetDir || "default location"}.`
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
        message: "Opened official GOG Galaxy download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for GOG Galaxy."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    const useGalaxy = config?.launchViaGalaxy ?? false;
    if (useGalaxy && game.appId) {
      return {
        protocolUri: `goggalaxy://openGameView/${game.appId}`
      };
    }
    if (game.exe) {
      return {
        executablePath: game.exe,
        args: game.launchArgs ? [game.launchArgs] : []
      };
    }
    if (game.appId) {
      return {
        protocolUri: `goggalaxy://openGameView/${game.appId}`
      };
    }
    return null;
  },

  getInstallUri(game): string | null {
    return game.appId ? `goggalaxy://openGameView/${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `goggalaxy://openGameView/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    const id = game.appName || game.appId;
    return id ? `https://www.gog.com/en/game/${id}` : null;
  },

  getFriendsListUri(): string | null {
    return "goggalaxy://openFriends";
  },

  getAchievementsUri(game): string | null {
    return game.appId ? `goggalaxy://openGameView/${game.appId}` : null;
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch(`https://embed.gog.com/games/ajax/filtered?mediaType=game&search=${encodeURIComponent(term)}`);
      if (!res.ok) return null;
      const data = await res.json();
      const product = data?.products?.[0];
      if (!product) return null;

      const rawImg = product.image ? (product.image.startsWith("//") ? `https:${product.image}` : product.image) : null;
      const capsule = rawImg ? `${rawImg}_product_card_v2_mobile_slider_639.jpg` : undefined;
      const hero = rawImg ? `${rawImg}_bg_crop_1920x1080.jpg` : undefined;
      const header = rawImg ? `${rawImg}.jpg` : undefined;

      const score = typeof product.rating === "number" ? Math.round((product.rating / 50) * 100) : undefined;
      const relDate = product.releaseDate ? new Date(product.releaseDate * 1000).toISOString().split("T")[0] : undefined;
      const relYear = product.releaseDate ? new Date(product.releaseDate * 1000).getFullYear() : undefined;

      return {
        name: product.title || term,
        releaseDate: relDate,
        releaseYear: relYear,
        developers: product.developers?.length ? product.developers : undefined,
        publishers: product.publishers?.length ? product.publishers : undefined,
        genres: product.genres || [],
        rating: score,
        criticScore: score,
        capsuleImage: capsule,
        heroImage: hero,
        headerImage: header,
        websiteUrl: product.url ? `https://www.gog.com${product.url}` : undefined
      };
    } catch {
      return null;
    }
  },

  async queryArtwork(query): Promise<PluginArtworkResult | null> {
    const meta = await this.queryMetadata!(query, {});
    if (!meta) return null;
    return {
      capsuleImage: meta.capsuleImage,
      heroImage: meta.heroImage,
      headerImage: meta.headerImage
    };
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files (x86)\\GOG Galaxy\\Games",
      "C:\\GOG Games",
      "D:\\GOG Games"
    ];
  }
};
