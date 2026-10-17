import type { UniversalPlugin, PlatformMetadata } from "./framework";
import {
  platformUniversalPlugin,
  PLATFORM_SPECS,
  getAllPlatformSpecs,
  createPlatformProviderAdapter,
} from "./platforms/universal";
import { metadataUniversalPlugin } from "./metadata/universal";
import { helixDbMetadataPlugin } from "./metadata/helix-db";
import {
  titleNormalizerPlugin,
  gamerpowerPlugin,
  storageAnalyzerPlugin,
  UTILITY_PLUGINS,
} from "./utility";
import { mediaUniversalPlugin, MEDIA_PLUGINS } from "./media";

/**
 * Core Universal Platform Plugin (`platform-universal`).
 */
export const CORE_PLATFORM_PLUGINS: UniversalPlugin[] = [platformUniversalPlugin];

/**
 * Per-platform adapter instances generated dynamically from `PLATFORM_SPECS` via `createPlatformProviderAdapter`
 * so UI views (Settings Launchers, Social Hub, Library Launch) can inspect or invoke any platform seamlessly.
 */
export const PLATFORM_PLUGINS: UniversalPlugin[] = getAllPlatformSpecs().map((spec) =>
  createPlatformProviderAdapter(spec)
);

/**
 * Core Metadata Plugins: HELIX Gaming DB (`metadata-helix`) & Universal Metadata Engine (`metadata-universal`).
 */
export const METADATA_PLUGINS: UniversalPlugin[] = [helixDbMetadataPlugin, metadataUniversalPlugin];

export {
  platformUniversalPlugin,
  helixDbMetadataPlugin,
  metadataUniversalPlugin,
  titleNormalizerPlugin,
  gamerpowerPlugin,
  storageAnalyzerPlugin,
  mediaUniversalPlugin,
  UTILITY_PLUGINS,
  MEDIA_PLUGINS,
};

/**
 * Master catalog of core Universal Plugins hosted in the application:
 * - `platform-universal`
 * - `metadata-universal`
 * - `utility-title-normalizer`, `utility-gamerpower`, `utility-storage`
 * - `media-universal`
 */
export const ALL_PLUGINS: UniversalPlugin[] = [
  ...CORE_PLATFORM_PLUGINS,
  ...METADATA_PLUGINS,
  ...UTILITY_PLUGINS,
  ...MEDIA_PLUGINS,
];

/**
 * Returns dynamic platform metadata mapping constructed directly from `PLATFORM_SPECS`.
 */
export function getAllPlatformMetadata(): Record<string, PlatformMetadata> {
  const map: Record<string, PlatformMetadata> = {};
  for (const spec of Object.values(PLATFORM_SPECS)) {
    map[spec.id] = {
      id: spec.id,
      label: spec.label,
      icon: spec.icon,
      color: spec.color,
      defaultUrl: spec.defaultStoreUrl,
      pluginId: `platform-${spec.id}`,
    };
  }
  return map;
}

/**
 * Resolves metadata (label, icon key, brand color, store url) for any platform or console dynamically.
 */
export function getPlatformMetadata(platformId?: string | null): PlatformMetadata {
  if (!platformId) {
    return {
      id: "unknown",
      label: "Unknown",
      icon: "standalone",
      color: "#94a3b8",
      defaultUrl: "",
      pluginId: "platform-universal",
    };
  }
  const all = getAllPlatformMetadata();
  const normalized = platformId.toLowerCase().trim();
  if (all[normalized]) {
    return all[normalized];
  }
  if (normalized === "other" || normalized === "custom") {
    return {
      id: "other",
      label: "Other",
      icon: "other",
      color: "#94a3b8",
      defaultUrl: "",
      pluginId: "platform-universal",
    };
  }
  return {
    id: normalized,
    label: normalized.charAt(0).toUpperCase() + normalized.slice(1),
    icon: normalized,
    color: "#94a3b8",
    defaultUrl: "",
    pluginId: "platform-universal",
  };
}
