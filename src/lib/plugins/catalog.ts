import type { UniversalPlugin, PlatformMetadata } from "./framework";

// Dedicated Platform Integration Plugins
import { platformSteamPlugin } from "./platforms/steam";
import { platformGogPlugin } from "./platforms/gog";
import { platformEpicPlugin } from "./platforms/epic";
import { platformItchPlugin } from "./platforms/itch";
import { platformXboxPlugin } from "./platforms/xbox";
import { platformEaPlugin } from "./platforms/ea";
import { platformUbisoftPlugin } from "./platforms/ubisoft";
import { platformBattlenetPlugin } from "./platforms/battlenet";
import { platformAmazonPlugin } from "./platforms/amazon";
import { platformRockstarPlugin } from "./platforms/rockstar";
import { platformStandalonePlugin } from "./platforms/standalone";
import { platformEmulationStationPlugin } from "./platforms/emulationstation";

// Dedicated Metadata Provider Plugins
import { steamStorePlugin } from "./metadata/steam-store";
import { steamgriddbPlugin } from "./metadata/steamgriddb";
import { igdbPlugin } from "./metadata/igdb";
import { itchStorePlugin } from "./metadata/itch-store";
import { howlongtobeatPlugin } from "./metadata/howlongtobeat";
import { pcgamingwikiPlugin } from "./metadata/pcgamingwiki";
import { protondbPlugin } from "./metadata/protondb";
import { gogCatalogPlugin } from "./metadata/gog-catalog";
import { epicStorePlugin } from "./metadata/epic-store";
import { thegamesdbPlugin } from "./metadata/thegamesdb";
import { screenscraperPlugin } from "./metadata/screenscraper";

// Dedicated Utility Plugins
import {
  gamerpowerPlugin,
  storageAnalyzerPlugin,
  UTILITY_PLUGINS
} from "./utility";

// Dedicated Media Enhancement Plugins
import {
  trailersPlayerPlugin,
  ambientSoundtracksPlugin,
  MEDIA_PLUGINS
} from "./media";

/** All registered first-class platform integration plugins. */
export const PLATFORM_PLUGINS: UniversalPlugin[] = [
  platformSteamPlugin,
  platformGogPlugin,
  platformEpicPlugin,
  platformItchPlugin,
  platformXboxPlugin,
  platformEaPlugin,
  platformUbisoftPlugin,
  platformBattlenetPlugin,
  platformAmazonPlugin,
  platformRockstarPlugin,
  platformStandalonePlugin,
  platformEmulationStationPlugin
];

/** All registered first-class metadata provider plugins. */
export const METADATA_PLUGINS: UniversalPlugin[] = [
  steamStorePlugin,
  steamgriddbPlugin,
  igdbPlugin,
  thegamesdbPlugin,
  screenscraperPlugin,
  itchStorePlugin,
  howlongtobeatPlugin,
  pcgamingwikiPlugin,
  protondbPlugin,
  gogCatalogPlugin,
  epicStorePlugin
];

export { UTILITY_PLUGINS, MEDIA_PLUGINS };

/** Master catalog of all plugins hosted directly in the application. */
export const ALL_PLUGINS: UniversalPlugin[] = [
  ...PLATFORM_PLUGINS,
  ...METADATA_PLUGINS,
  ...UTILITY_PLUGINS,
  ...MEDIA_PLUGINS
];

const DEFAULT_PLATFORM_LABELS: Record<string, string> = {
  steam: "Steam",
  gog: "GOG Galaxy",
  epic: "Epic Games",
  itch: "itch.io",
  xbox: "Xbox",
  ea: "EA App",
  ubisoft: "Ubisoft Connect",
  battlenet: "Battle.net",
  amazon: "Amazon Games",
  rockstar: "Rockstar Games",
  standalone: "Standalone",
  emulationstation: "EmulationStation",
  other: "Other"
};

/**
 * Returns dynamic platform metadata mapping dynamically constructed from all registered platform plugins.
 */
export function getAllPlatformMetadata(): Record<string, PlatformMetadata> {
  const map: Record<string, PlatformMetadata> = {};
  for (const plugin of PLATFORM_PLUGINS) {
    const id = plugin.platformId || plugin.id.replace("platform-", "");
    const label = plugin.platformLabel || DEFAULT_PLATFORM_LABELS[id] || plugin.name.replace(/ (Platform )?Integration.*$/i, "").trim();
    map[id] = {
      id,
      label,
      icon: plugin.icon || "🎮",
      color: plugin.platformColor || "#94a3b8",
      defaultUrl: plugin.defaultStoreUrl || "",
      pluginId: plugin.id
    };
  }
  return map;
}

/**
 * Resolves metadata (label, icon, brand color, store url) for any platform dynamically.
 */
export function getPlatformMetadata(platformId?: string | null): PlatformMetadata {
  if (!platformId) {
    return {
      id: "unknown",
      label: "Unknown",
      icon: "🎮",
      color: "#94a3b8",
      defaultUrl: "",
      pluginId: ""
    };
  }
  const all = getAllPlatformMetadata();
  const normalized = platformId.toLowerCase().trim();
  if (all[normalized]) {
    return all[normalized];
  }
  if (normalized === "other") {
    return {
      id: "other",
      label: "Other",
      icon: "⚙",
      color: "#94a3b8",
      defaultUrl: "",
      pluginId: "platform-standalone"
    };
  }
  return {
    id: normalized,
    label: normalized.charAt(0).toUpperCase() + normalized.slice(1),
    icon: "🎮",
    color: "#94a3b8",
    defaultUrl: "",
    pluginId: `platform-${normalized}`
  };
}

