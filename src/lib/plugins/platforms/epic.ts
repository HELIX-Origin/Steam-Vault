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

export const platformEpicPlugin: UniversalPlugin = {
  id: "platform-epic",
  name: "Epic Games Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games from Epic Games .item manifests and integrated open-source Legendary CLI engine.",
  homepage: "https://store.epicgames.com",
  features: ["discovery", "launch", "legendary_cli", "offline_auth", "eos_overlay", "social", "achievements"],

  // Dynamic platform metadata
  icon: "▲",
  platformId: "epic",
  platformColor: "#facc15",
  defaultStoreUrl: "https://store.epicgames.com",

  // Provider launcher capabilities
  launcherBinaryNames: ["EpicGamesLauncher.exe", "legendary.exe", "legendary"],
  wingetPackageId: "EpicGames.EpicGamesLauncher",
  downloadUrl: "https://store.epicgames.com/download",
  alternativeUrl: "https://github.com/legendary-gl/legendary/releases",
  alternativeLabel: "Download Legendary CLI ↗",
  supportsSilentInstall: true,
  supportsCustomInstallPath: true,

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customBinaryPath",
          label: "Optional Custom Epic Games Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'EpicGamesLauncher.exe') or absolute path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "EpicGamesLauncher.exe (leave blank to autodetect)"
        },
        {
          key: "useLegendary",
          label: "Use Legendary CLI as Alternate Provider Method",
          type: "boolean",
          default: false,
          description: "Use open-source Legendary CLI in place of the official Epic Games Launcher. Autodetects the 'legendary' binary from PATH or tools directory by default."
        },
        {
          key: "customLegendaryPath",
          label: "Optional Custom Legendary Binary / Path",
          type: "text",
          description: "Optional binary name (e.g. 'legendary') or absolute path in case autodetection fails. Leave blank to autodetect automatically.",
          placeholder: "legendary (leave blank to autodetect)"
        },
        {
          key: "enableEosOverlay",
          label: "Enable Epic Online Services (EOS) Overlay",
          type: "boolean",
          default: false,
          description: "Injects EOS overlay for multiplayer invites and Epic achievements."
        }
      ]
    };
  },

  async testConnection(config): Promise<TestConnectionResult> {
    const detection = await this.detectLauncher?.(config);
    if (detection?.installed) {
      return {
        success: true,
        message: `Epic Games integration active via ${detection.alternativeMethod || detection.method} at ${detection.executablePath ?? "system"}.`
      };
    }
    return {
      success: false,
      message: "Neither Epic Games Launcher nor Legendary CLI detected on PATH or standard directories."
    };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    const useLegendary = config?.useLegendary === true;
    const customLegendary = config?.customLegendaryPath?.trim();
    const customBinary = config?.customBinaryPath?.trim();

    // 1. If configured to use Legendary CLI
    if (useLegendary || customLegendary) {
      const legDetected = await api.checkBinaryInstalled(["legendary.exe", "legendary"], customLegendary || null);
      if (legDetected) {
        return {
          installed: true,
          executablePath: legDetected,
          detectedBinary: "legendary",
          method: customLegendary ? "custom_path" : "cli_tool",
          alternativeMethod: "Legendary CLI & EOS Service",
          downloadUrl: this.downloadUrl,
          supportsSilentInstall: true,
          wingetPackageId: this.wingetPackageId,
        };
      }
    }

    // 2. Official Epic Games Launcher binary detection
    const custom = customBinary || null;
    const detected = await api.checkBinaryInstalled(this.launcherBinaryNames ?? ["EpicGamesLauncher.exe"], custom);
    if (detected) {
      const isLeg = detected.toLowerCase().includes("legendary");
      return {
        installed: true,
        executablePath: detected,
        detectedBinary: detected,
        method: custom ? "custom_path" : (isLeg ? "cli_tool" : "path_binary"),
        alternativeMethod: isLeg ? "Legendary CLI & EOS Service" : undefined,
        downloadUrl: this.downloadUrl,
        supportsSilentInstall: true,
        wingetPackageId: this.wingetPackageId,
      };
    }

    // 3. Legendary fallback if available
    const legFallback = await api.checkBinaryInstalled(["legendary.exe", "legendary"], null);
    if (legFallback) {
      return {
        installed: true,
        executablePath: legFallback,
        detectedBinary: "legendary",
        method: "cli_tool",
        alternativeMethod: "Legendary CLI & EOS Service",
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

  async installLauncher(options?: LauncherInstallOptions, config?: Record<string, any>): Promise<LauncherInstallResult> {
    const useLegendary = config?.useLegendary === true;

    // Direct standalone Legendary CLI installation
    if (useLegendary) {
      try {
        const res = await api.installLegendary();
        return {
          success: res.installed,
          message: "Legendary CLI successfully installed and initialized.",
          installedPath: res.executablePath ?? null,
        };
      } catch (err: any) {
        return {
          success: false,
          message: `Legendary CLI installation failed: ${err?.message || err}`
        };
      }
    }

    // WinGet silent install for official Epic Games Launcher
    const wingetReady = await api.isWingetAvailable();
    if (wingetReady && this.wingetPackageId) {
      try {
        await api.installWingetPackage(this.wingetPackageId, options?.targetDir || null);
        return {
          success: true,
          message: `Epic Games Launcher successfully installed via winget into ${options?.targetDir || "default location"}.`
        };
      } catch (err: any) {
        // Fall back to installing Legendary CLI or opening web download
        try {
          const res = await api.installLegendary();
          return {
            success: res.installed,
            message: `WinGet failed (${err?.message || err}), but installed built-in Legendary CLI as alternate provider.`,
            installedPath: res.executablePath ?? null,
          };
        } catch {
          if (this.downloadUrl) {
            await api.openExternalUrl(this.downloadUrl);
          }
          return {
            success: false,
            message: `WinGet installation failed: ${err?.message || err}. Opened official download page.`
          };
        }
      }
    }

    if (this.downloadUrl) {
      await api.openExternalUrl(this.downloadUrl);
      return {
        success: true,
        message: "Opened official Epic Games download page in web browser."
      };
    }

    return {
      success: false,
      message: "No download URL or silent installer available for Epic Games."
    };
  },

  getLauncherDownloadUrl(): string | null {
    return this.downloadUrl ?? null;
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    const customPath = config?.customLegendaryPath?.trim();
    const useLegendary = config?.useLegendary === true;
    const identifier = game.appName || (game.appId !== undefined ? game.appId.toString() : "");

    if (useLegendary && identifier) {
      return {
        executablePath: customPath || "legendary",
        args: ["launch", identifier]
      };
    }

    // Default: Official Epic Games Launcher
    const appId = game.appName || game.appId;
    if (appId) {
      return {
        protocolUri: `com.epicgames.launcher://apps/${appId}?action=launch&silent=true`
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
    return game.appId ? `com.epicgames.launcher://apps/${game.appId}?action=install` : null;
  },

  getUninstallUri(game): string | null {
    return game.appId ? `com.epicgames.launcher://apps/${game.appId}?action=uninstall` : null;
  },

  getStorePageUri(game): string | null {
    const id = game.appName || game.appId;
    return id ? `https://store.epicgames.com/p/${id}` : null;
  },

  getFriendsListUri(): string | null {
    return "com.epicgames.launcher://friends";
  },

  getAchievementsUri(game): string | null {
    const id = game.appName || game.appId;
    return id ? `com.epicgames.launcher://achievements/${id}` : null;
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch("https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US");
      if (!res.ok) return null;
      const data = await res.json();
      const elements: any[] = data?.data?.Catalog?.searchStore?.elements || [];
      const game = elements.find((e: any) => e.title?.toLowerCase().includes(term.toLowerCase()));
      if (!game) return null;

      const tallImg = game.keyImages?.find((img: any) => img.type === "OfferImageTall" || img.type === "Thumbnail")?.url;
      const wideImg = game.keyImages?.find((img: any) => img.type === "OfferImageWide" || img.type === "DieselStoreFrontWide")?.url;

      const devAttr = game.customAttributes?.find((a: any) => a.key === "developerName")?.value;
      const pubAttr = game.seller?.name || game.customAttributes?.find((a: any) => a.key === "publisherName")?.value;

      const relDate = game.effectiveDate?.split("T")[0];
      const relYear = game.effectiveDate ? new Date(game.effectiveDate).getFullYear() : undefined;

      const slug = game.productSlug || game.urlSlug;

      return {
        name: game.title || term,
        shortDescription: game.description,
        detailedDescription: game.description,
        releaseDate: relDate,
        releaseYear: relYear,
        developers: devAttr ? [devAttr] : undefined,
        publishers: pubAttr ? [pubAttr] : undefined,
        categories: (game.categories || []).map((c: any) => c.path).filter(Boolean),
        capsuleImage: tallImg,
        heroImage: wideImg,
        headerImage: wideImg,
        websiteUrl: slug ? `https://store.epicgames.com/p/${slug}` : undefined
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
      "C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests",
      "C:\\Program Files\\Epic Games",
      "D:\\Epic Games"
    ];
  }
};

