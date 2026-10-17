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
import type { PlatformFriend, PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";

export interface PlatformSpec {
  id: string;
  name: string;
  label: string;
  icon: string;
  color: string;
  defaultStoreUrl: string;
  launcherBinaryNames: string[];
  wingetPackageId?: string;
  downloadUrl: string;
  alternativeUrl?: string;
  alternativeLabel?: string;
  supportsSilentInstall: boolean;
  supportsCustomInstallPath: boolean;
  knownFolders: string[];
  friendsUri?: string;
}

export const PLATFORM_SPECS: Record<string, PlatformSpec> = {
  steam: {
    id: "steam",
    name: "Steam",
    label: "Steam",
    icon: "steam",
    color: "#38bdf8",
    defaultStoreUrl: "https://store.steampowered.com",
    launcherBinaryNames: ["steam.exe", "steam"],
    wingetPackageId: "Valve.Steam",
    downloadUrl: "https://store.steampowered.com/about/",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files (x86)\\Steam", "C:\\Program Files\\Steam", "%APPDATA%\\Steam"],
    friendsUri: "steam://friends/status",
  },
  gog: {
    id: "gog",
    name: "GOG Galaxy",
    label: "GOG Galaxy",
    icon: "gog",
    color: "#a855f7",
    defaultStoreUrl: "https://www.gog.com",
    launcherBinaryNames: ["GalaxyClient.exe", "GalaxyCommunication.exe"],
    wingetPackageId: "GOG.Galaxy",
    downloadUrl: "https://www.gog.com/galaxy",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files (x86)\\GOG Galaxy\\Games", "C:\\GOG Games"],
    friendsUri: "goggalaxy://openFriends",
  },
  epic: {
    id: "epic",
    name: "Epic Games",
    label: "Epic Games",
    icon: "epic",
    color: "#facc15",
    defaultStoreUrl: "https://store.epicgames.com",
    launcherBinaryNames: ["EpicGamesLauncher.exe", "legendary.exe", "legendary"],
    wingetPackageId: "EpicGames.EpicGamesLauncher",
    downloadUrl: "https://store.epicgames.com/download",
    alternativeUrl: "https://github.com/legendary-gl/legendary/releases",
    alternativeLabel: "Download Legendary CLI ↗",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files\\Epic Games", "C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests"],
    friendsUri: "com.epicgames.launcher://friends",
  },
  itch: {
    id: "itch",
    name: "itch.io",
    label: "itch.io",
    icon: "itch",
    color: "#fb7185",
    defaultStoreUrl: "https://itch.io",
    launcherBinaryNames: ["itch.exe", "itch", "butler.exe", "butler"],
    wingetPackageId: "ItchIo.Itch",
    downloadUrl: "https://itch.io/app",
    alternativeUrl: "https://itchio.itch.io/butler",
    alternativeLabel: "itch.io Butler CLI Docs ↗",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["%APPDATA%\\itch\\apps", "%LOCALAPPDATA%\\itch"],
  },
  xbox: {
    id: "xbox",
    name: "Xbox",
    label: "Xbox",
    icon: "xbox",
    color: "#22c55e",
    defaultStoreUrl: "https://www.xbox.com/games/all-games",
    launcherBinaryNames: ["XboxPcApp.exe", "XboxApp.exe", "GamingServices.exe"],
    wingetPackageId: "Microsoft.GamingApp",
    downloadUrl: "https://www.xbox.com/apps/xbox-app-for-pc",
    supportsSilentInstall: true,
    supportsCustomInstallPath: false,
    knownFolders: ["C:\\XboxGames", "C:\\Program Files\\ModifiableWindowsApps"],
    friendsUri: "ms-xbox-overlay:friends",
  },
  ea: {
    id: "ea",
    name: "EA App",
    label: "EA App",
    icon: "ea",
    color: "#f97316",
    defaultStoreUrl: "https://www.ea.com/games",
    launcherBinaryNames: ["EADesktop.exe", "EALauncher.exe", "Origin.exe"],
    wingetPackageId: "ElectronicArts.EADesktop",
    downloadUrl: "https://www.ea.com/ea-app",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files\\EA Games", "C:\\Program Files (x86)\\Origin Games"],
    friendsUri: "origin2://friends/show",
  },
  ubisoft: {
    id: "ubisoft",
    name: "Ubisoft Connect",
    label: "Ubisoft Connect",
    icon: "ubisoft",
    color: "#06b6d4",
    defaultStoreUrl: "https://store.ubisoft.com",
    launcherBinaryNames: ["UbisoftConnect.exe", "upc.exe", "Uplay.exe"],
    wingetPackageId: "Ubisoft.Connect",
    downloadUrl: "https://ubisoftconnect.com",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files (x86)\\Ubisoft\\Ubisoft Game Launcher\\games"],
    friendsUri: "uplay://friends",
  },
  battlenet: {
    id: "battlenet",
    name: "Battle.net",
    label: "Battle.net",
    icon: "battlenet",
    color: "#3b82f6",
    defaultStoreUrl: "https://shop.battle.net",
    launcherBinaryNames: ["Battle.net.exe", "Battle.net Launcher.exe"],
    wingetPackageId: "Blizzard.BattleNet",
    downloadUrl: "https://www.blizzard.com/download/",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files (x86)\\Battle.net"],
    friendsUri: "battlenet://friends",
  },
  amazon: {
    id: "amazon",
    name: "Amazon Games",
    label: "Amazon Games",
    icon: "amazon",
    color: "#f59e0b",
    defaultStoreUrl: "https://gaming.amazon.com",
    launcherBinaryNames: ["Amazon Games.exe", "nile.exe", "nile"],
    wingetPackageId: "Amazon.Games",
    downloadUrl: "https://gaming.amazon.com/amazon-games-app",
    alternativeUrl: "https://github.com/imLinguin/nile/releases",
    alternativeLabel: "Download Nile CLI ↗",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Amazon Games\\Library", "%LOCALAPPDATA%\\Amazon Games"],
  },
  rockstar: {
    id: "rockstar",
    name: "Rockstar Games",
    label: "Rockstar Games",
    icon: "rockstar",
    color: "#eab308",
    defaultStoreUrl: "https://store.rockstargames.com",
    launcherBinaryNames: ["Launcher.exe", "LauncherPatcher.exe", "RockstarService.exe"],
    wingetPackageId: "RockstarGames.Launcher",
    downloadUrl: "https://socialclub.rockstargames.com/rockstar-games-launcher",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\Program Files\\Rockstar Games"],
  },
  emulationstation: {
    id: "emulationstation",
    name: "EmulationStation",
    label: "EmulationStation",
    icon: "emulationstation",
    color: "#ec4899",
    defaultStoreUrl: "https://es-de.org",
    launcherBinaryNames: ["es-de.exe", "emulationstation.exe", "retroarch.exe"],
    wingetPackageId: "Libretro.RetroArch",
    downloadUrl: "https://es-de.org/#Download",
    alternativeUrl: "https://www.retroarch.com/?page=platforms",
    alternativeLabel: "Download RetroArch ↗",
    supportsSilentInstall: true,
    supportsCustomInstallPath: true,
    knownFolders: ["C:\\ROMs", "D:\\ROMs", "E:\\ROMs", "E:\\Games\\ROMs"],
  },
  standalone: {
    id: "standalone",
    name: "Standalone Games",
    label: "Standalone",
    icon: "standalone",
    color: "#94a3b8",
    defaultStoreUrl: "",
    launcherBinaryNames: [],
    downloadUrl: "",
    supportsSilentInstall: false,
    supportsCustomInstallPath: false,
    knownFolders: ["C:\\Games", "D:\\Games", "E:\\Games"],
  },
};

/**
 * Global helper returning all supported platform specifications.
 */
export function getAllPlatformSpecs(): PlatformSpec[] {
  return Object.values(PLATFORM_SPECS);
}

/**
 * Global helper resolving a platform specification by ID (normalizing `"other"` / `"custom"` to `"standalone"`).
 */
export function getPlatformSpec(platformId?: string | null): PlatformSpec | undefined {
  if (!platformId) return undefined;
  const norm = platformId.toLowerCase().trim().replace(/^platform-/, "");
  if (PLATFORM_SPECS[norm]) return PLATFORM_SPECS[norm];
  if (norm === "other" || norm === "custom") return PLATFORM_SPECS.standalone;
  return undefined;
}

/**
 * Global helper detecting whether a specific platform launcher or CLI alternative is installed on the host.
 */
export async function detectPlatformLauncher(
  platformId: string,
  config?: Record<string, any>
): Promise<LauncherDetectionResult> {
  const spec = getPlatformSpec(platformId);
  if (!spec) {
    return { installed: false, method: "not_installed" };
  }

  if (spec.id === "standalone") {
    return {
      installed: true,
      method: "standalone",
      alternativeMethod: "Direct OS Executable Runner",
      supportsSilentInstall: false,
    };
  }

  if (spec.id === "emulationstation") {
    const custom = config?.customBinaryPath?.trim() || null;
    const detected = await api.checkBinaryInstalled(spec.launcherBinaryNames, custom);
    return {
      installed: true,
      executablePath: detected || "Native Built-in ROM & Emulator Scanner",
      detectedBinary: detected || "built-in",
      method: detected ? (custom ? "custom_path" : "path_binary") : "cli_tool",
      alternativeMethod: "Built-in Multi-Drive ROM & Archive Engine",
      downloadUrl: spec.downloadUrl,
      supportsSilentInstall: spec.supportsSilentInstall,
      wingetPackageId: spec.wingetPackageId,
    };
  }

  const custom = config?.customBinaryPath?.trim() || null;
  if (spec.launcherBinaryNames.length > 0) {
    const detected = await api.checkBinaryInstalled(spec.launcherBinaryNames, custom);
    if (detected) {
      const lower = detected.toLowerCase();
      const isAltCli = lower.includes("legendary") || lower.includes("butler") || lower.includes("nile");
      return {
        installed: true,
        executablePath: detected,
        detectedBinary: detected,
        method: custom ? "custom_path" : isAltCli ? "cli_tool" : "path_binary",
        alternativeMethod: isAltCli ? `CLI Integration (${detected})` : undefined,
        downloadUrl: spec.downloadUrl,
        supportsSilentInstall: spec.supportsSilentInstall,
        wingetPackageId: spec.wingetPackageId,
      };
    }
  }

  if (spec.id === "steam") {
    try {
      const steamInstall = await api.detectSteam();
      if (steamInstall?.root) {
        return {
          installed: true,
          executablePath: steamInstall.root,
          detectedBinary: "steam.exe",
          method: "system_store",
          downloadUrl: spec.downloadUrl,
          supportsSilentInstall: spec.supportsSilentInstall,
          wingetPackageId: spec.wingetPackageId,
        };
      }
    } catch {
      // Fall through
    }
  }

  return {
    installed: false,
    downloadUrl: spec.downloadUrl,
    supportsSilentInstall: spec.supportsSilentInstall,
    wingetPackageId: spec.wingetPackageId,
    method: "not_installed",
  };
}

/**
 * Global helper installing a platform launcher silently via WinGet or opening its official download URL.
 */
export async function installPlatformLauncher(
  platformId: string,
  options?: LauncherInstallOptions
): Promise<LauncherInstallResult> {
  const spec = getPlatformSpec(platformId);
  if (!spec) {
    return { success: false, message: `Unknown platform '${platformId}'.` };
  }

  const wingetReady = await api.isWingetAvailable();
  if (wingetReady && spec.wingetPackageId) {
    try {
      await api.installWingetPackage(spec.wingetPackageId, options?.targetDir || null);
      return {
        success: true,
        message: `${spec.name} successfully installed via WinGet into ${options?.targetDir || "default location"}.`,
      };
    } catch (err: any) {
      if (spec.downloadUrl) {
        await api.openExternalUrl(spec.downloadUrl);
      }
      return {
        success: false,
        message: `WinGet installation failed: ${err?.message || err}. Opened official download page.`,
      };
    }
  }

  if (spec.downloadUrl) {
    await api.openExternalUrl(spec.downloadUrl);
    return {
      success: true,
      message: `Opened official ${spec.name} download page in web browser.`,
    };
  }

  return {
    success: false,
    message: `No external installer required for ${spec.name}.`,
  };
}

/**
 * Global helper resolving the launch specification (protocol URI or direct executable) for any game.
 */
export function getPlatformLaunchSpec(
  platformId: string,
  game: { appId?: number | string; name?: string; exe?: string | null; installPath?: string | null; launchArgs?: string | null },
  config?: Record<string, any>
): PlatformLaunchSpec | null {
  const norm = (platformId || "standalone").toLowerCase().trim();
  if (norm === "steam" && game.appId && Number(game.appId) > 0 && Number(game.appId) < 50_000_000) {
    const preferProtocol = config?.launchViaProtocol ?? true;
    if (preferProtocol || !game.exe) {
      return { protocolUri: `steam://rungameid/${game.appId}` };
    }
  }

  if (game.exe) {
    return {
      executablePath: game.exe,
      args: game.launchArgs ? [game.launchArgs] : [],
    };
  }

  if (norm === "steam" && game.appId) {
    return { protocolUri: `steam://rungameid/${game.appId}` };
  }

  return null;
}

/**
 * Global helper returning protocol URIs for install, uninstall, store page, friends list, and achievements.
 */
export function getPlatformUris(
  platformId: string,
  game?: { appId?: number | string; name?: string }
): {
  installUri: string | null;
  uninstallUri: string | null;
  storePageUri: string | null;
  friendsListUri: string | null;
  achievementsUri: string | null;
} {
  const spec = getPlatformSpec(platformId);
  const norm = spec?.id || "standalone";
  const appId = game?.appId;
  const isNativeSteam = norm === "steam" && appId && Number(appId) > 0 && Number(appId) < 50_000_000;

  return {
    installUri: isNativeSteam ? `steam://install/${appId}` : null,
    uninstallUri: isNativeSteam ? `steam://uninstall/${appId}` : null,
    storePageUri: isNativeSteam
      ? `https://store.steampowered.com/app/${appId}`
      : spec?.defaultStoreUrl || null,
    friendsListUri: spec?.friendsUri || null,
    achievementsUri: isNativeSteam ? `steam://url/SteamIDControlPage/${appId}` : null,
  };
}

/**
 * Global helper querying active friends list for a given platform.
 */
export async function getPlatformFriendsList(_platformId: string): Promise<PlatformFriend[]> {
  return [];
}

/**
 * Creates a platform-scoped `UniversalPlugin` adapter delegating to the Universal Platform Plugin's global helpers.
 * Used when callers inspect per-platform launcher status (`getProviderPlugin(platformId)` or `PLATFORM_PLUGINS`).
 */
export function createPlatformProviderAdapter(spec: PlatformSpec): UniversalPlugin {
  return {
    id: `platform-${spec.id}`,
    name: `${spec.name} Integration`,
    version: "1.0.0",
    pluginType: "platform",
    author: "HELIX Origin",
    description: `Universal Platform Integration handler for ${spec.name}.`,
    homepage: spec.defaultStoreUrl || "https://github.com/HELIX-Origin/Universal-Game-Launcher",
    features: ["discovery", "launch", "metadata"],
    icon: spec.icon,
    platformId: spec.id,
    platformLabel: spec.label,
    platformColor: spec.color,
    defaultStoreUrl: spec.defaultStoreUrl,
    launcherBinaryNames: spec.launcherBinaryNames,
    wingetPackageId: spec.wingetPackageId,
    downloadUrl: spec.downloadUrl,
    alternativeUrl: spec.alternativeUrl,
    alternativeLabel: spec.alternativeLabel,
    supportsSilentInstall: spec.supportsSilentInstall,
    supportsCustomInstallPath: spec.supportsCustomInstallPath,

    detectLauncher: (config) => detectPlatformLauncher(spec.id, config),
    installLauncher: (options) => installPlatformLauncher(spec.id, options),
    getLauncherDownloadUrl: () => spec.downloadUrl || null,
    getLaunchSpec: (game, config) => getPlatformLaunchSpec(spec.id, game, config),
    getInstallUri: (game) => getPlatformUris(spec.id, game).installUri,
    getUninstallUri: (game) => getPlatformUris(spec.id, game).uninstallUri,
    getStorePageUri: (game) => getPlatformUris(spec.id, game).storePageUri,
    getFriendsListUri: () => spec.friendsUri || null,
    getFriendsList: () => getPlatformFriendsList(spec.id),
    getAchievementsUri: (game) => getPlatformUris(spec.id, game).achievementsUri,
    getDefaultKnownFolders: () => spec.knownFolders,
  };
}

/**
 * Universal Platform Plugin (`platform-universal`) consolidating all 12 PC storefronts,
 * EmulationStation ROM directories, and standalone game libraries into a single core plugin.
 */
export const platformUniversalPlugin: UniversalPlugin = {
  id: "platform-universal",
  name: "Universal Multi-Platform Discovery & Launch Engine",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description:
    "Unified multi-platform game discovery, launcher detection, protocol URI routing, and direct executable runner for Steam, GOG, Epic/Legendary, itch.io, Xbox, EA, Ubisoft, Battle.net, Amazon, Rockstar, EmulationStation, and Standalone games.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: [
    "discovery",
    "launch",
    "library_vdf",
    "legendary_cli",
    "emulation_roms",
    "drm_free",
    "social",
    "achievements",
  ],
  icon: "all",
  platformId: "universal",
  platformColor: "#38bdf8",
  defaultStoreUrl: "https://store.steampowered.com",

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "autoScanAllDrives",
          label: "Automatically Scan All Connected Physical Drives",
          type: "boolean",
          default: true,
          description: "Scan all non-cloud mounted drives for Steam, GOG, Epic, itch.io, Xbox, EA, Ubisoft, Battle.net, Amazon, Rockstar, EmulationStation ROMs, and standalone games."
        },
        {
          key: "launchViaProtocol",
          label: "Prefer Native Store Protocol URIs (steam://, com.epicgames.launcher://)",
          type: "boolean",
          default: true,
          description: "Preserve store overlays and cloud saves when launching store-managed titles."
        },
        {
          key: "romsPath",
          label: "Custom EmulationStation ROMs Root Directory",
          type: "text",
          default: "C:\\ROMs",
          description: "Optional primary ROMs folder containing console subdirectories (gc, wii, switch, ps2, snes, etc.).",
          placeholder: "C:\\ROMs"
        },
        {
          key: "normalizeTitles",
          label: "Automatically Normalize Imported Titles",
          type: "boolean",
          default: true,
          description: "Clean ROM region tags, invert trailing articles, and inspect RPG Maker / Ren'Py / Unity manifests during import."
        }
      ]
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    return {
      success: true,
      message: `Universal Platform Engine active (${Object.keys(PLATFORM_SPECS).length} platform specifications loaded).`,
    };
  },

  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    return detectPlatformLauncher("steam", config);
  },

  async installLauncher(options?: LauncherInstallOptions): Promise<LauncherInstallResult> {
    return installPlatformLauncher("steam", options);
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    return getPlatformLaunchSpec("steam", game, config);
  },

  async queryArtwork(query): Promise<PluginArtworkResult | null> {
    const appId = Number(query.appId);
    if (appId > 0 && appId < 50_000_000) {
      return {
        capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_600x900_2x.jpg`,
        heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`,
        logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`,
        iconImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`,
      };
    }
    return null;
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const appId = Number(query.appId);
    if (appId > 0 && appId < 50_000_000) {
      try {
        const meta = await api.refreshGameMetadata(appId, false);
        if (meta) {
          return {
            name: meta.name || undefined,
            shortDescription: meta.shortDescription || undefined,
            releaseDate: meta.releaseDate || undefined,
            releaseYear: meta.releaseYear || undefined,
            developers: meta.developers,
            publishers: meta.publishers,
            genres: meta.genres,
            categories: meta.categories,
            tags: meta.tags,
            rating: meta.userScore || undefined,
            headerImage: meta.headerImage || undefined,
            capsuleImage: meta.capsuleImage || undefined,
            heroImage: meta.heroImage || undefined,
            logoImage: meta.logoImage || undefined,
            iconImage: meta.iconImage || undefined,
            screenshots: meta.screenshots,
            movies: meta.movies,
            soundtracks: meta.soundtracks,
          };
        }
      } catch {
        // Fall through
      }
    }
    return null;
  },
};
