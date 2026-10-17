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

export const platformSteamPlugin: UniversalPlugin = {
  id: "platform-steam",
  name: "Steam Platform Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Steam libraries, appmanifest files, and custom library folders across all mounted drives.",
  homepage: "https://store.steampowered.com",
  features: ["discovery", "launch", "library_vdf", "cloud_saves", "store_cdn", "social", "achievements"],

  // Dynamic platform metadata
  icon: "♨",
  platformId: "steam",
  platformColor: "#38bdf8",
  defaultStoreUrl: "https://store.steampowered.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["steam.exe", "steam"],
  wingetPackageId: "Valve.Steam",
  downloadUrl: "https://store.steampowered.com/about/",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Steam Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'steam') or absolute executable path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "steam (leave blank to autodetect)"
        },
        {
          key: "customSteamRoot",
          label: "Custom Steam Installation Directory",
          type: "text",
          description: "Optional custom root directory for Steam (defaults to auto-detected path).",
          placeholder: "C:\\Program Files (x86)\\Steam"
        },
        {
          key: "autoScanLibraries",
          label: "Scan All Connected Drive Partitions",
          type: "boolean",
          default: true,
          description: "Automatically scan all secondary drives for SteamLibrary folders."
        },
        {
          key: "launchViaProtocol",
          label: "Prefer Native steam:// URI Protocol",
          type: "boolean",
          default: true,
          description: "Launch games via the official steam://rungameid/{appId} protocol to preserve Steam overlay and achievements."
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    try {
      const detection = await this.detectLauncher?.(config);
      if (detection?.installed) {
        return {
          success: true,
          message: `Steam detected (${detection.method}) at ${detection.executablePath ?? "system"}.`
        };
      }
      return { success: false, message: "Steam installation not found on standard paths or PATH." };
    } catch (e: any) {
      return { success: false, message: `Detection error: ${e?.message || e}` };
    }
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const custom = config?.customBinaryPath?.trim() || config?.customSteamRoot?.trim();
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["steam.exe", "steam"], custom || null);
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

    try {
      const steamInstall = await api.detectSteam();
      if (steamInstall?.root) {
        return {
          installed: true,
          executablePath: steamInstall.root,
          detectedBinary: "steam.exe",
          method: "system_store",
          downloadUrl: this.downloadUrl,
          supportsSilentInstall: true,
          wingetPackageId: this.wingetPackageId,
        };
      }
    } catch {
      // Fall through
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
          message: `Steam successfully installed via winget into ${options?.targetDir || "default location"}.`
        };
      } catch (err: any) {
        // Fall back to opening official download page
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
        message: "Opened official Steam download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Steam."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    if (!game.appId) return null;
    const preferProtocol = config?.launchViaProtocol ?? true;
    if (preferProtocol) {
      return {
        protocolUri: `steam://rungameid/${game.appId}`
      };
    }
    if (game.exe) {
      return {
        executablePath: game.exe,
        args: game.launchArgs ? [game.launchArgs] : []
      };
    }
    return {
      protocolUri: `steam://rungameid/${game.appId}`
    };
  },

  getInstallUri(game): string | null {
    return game.appId ? `steam://install/${game.appId}` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `steam://uninstall/${game.appId}` : null;
  },

  getStorePageUri(game): string | null {
    return game.appId ? `https://store.steampowered.com/app/${game.appId}` : null;
  },

  getFriendsListUri(): string | null {
    return "steam://friends/status";
  },

  getAchievementsUri(game): string | null {
    return game.appId ? `steam://url/SteamIDControlPage/${game.appId}` : null;
  },

  async queryArtwork(query): Promise<PluginArtworkResult | null> {
    const appId = query.appId;
    if (!appId) return null;
    return {
      capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/capsule_616x353.jpg`,
      heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`,
      logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`,
      iconImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`
    };
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const appId = query.appId;
    if (!appId) return null;
    const cc = (config?.countryCode || "us").trim();
    const lang = (config?.language || "english").trim();

    try {
      const res = await fetch(`https://store.steampowered.com/api/appdetails?appids=${appId}&cc=${cc}&l=${lang}`);
      if (!res.ok) return null;
      const data = await res.json();
      const entry = data[appId];
      if (!entry?.success || !entry?.data) return null;

      const d = entry.data;
      return {
        name: d.name,
        shortDescription: d.short_description,
        detailedDescription: d.detailed_description,
        releaseDate: d.release_date?.date,
        developers: d.developers || [],
        publishers: d.publishers || [],
        genres: (d.genres || []).map((g: any) => g.description),
        categories: (d.categories || []).map((c: any) => c.description),
        criticScore: d.metacritic?.score,
        capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/capsule_616x353.jpg`,
        heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`,
        logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`,
        headerImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`,
        websiteUrl: d.website || `https://store.steampowered.com/app/${appId}`
      };
    } catch {
      return null;
    }
  },

  getDefaultKnownFolders(): string[] {
    return [
      "C:\\Program Files (x86)\\Steam",
      "C:\\Program Files\\Steam",
      "%APPDATA%\\Steam"
    ];
  }
};

