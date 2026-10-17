// Typed wrappers around the Tauri IPC commands.
// Tauri maps camelCase JS arguments to snake_case Rust parameters automatically.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppMeta,
  ApplyResult,
  ArtworkOutcome,
  ArtworkSlotId,
  CacheResetReport,
  CacheStats,
  Collection,
  EosOverlayStatus,
  FeaturedStoreItem,
  Filter,
  FilterPreset,
  FreeGamePromotion,
  Game,
  GameArtwork,
  ImportResult,
  LauncherStatus,
  LegendaryAuthStatus,
  ManagedDirectories,
  MetadataSyncResult,
  MoveOutcome,
  MovePreview,
  OwnedGame,
  OwnedLibrary,
  PatchNote,
  PlatformGame,
  PlatformScan,
  PluginInfo,
  ProviderInfo,
  ScaffoldReport,
  Settings,
  SteamInstall,
  SteamSearchResult,
  StorageReport,
  StoreDeal,
} from "./types";

export const api = {
  getProviders: () => invoke<ProviderInfo[]>("get_providers"),
  detectSteam: () => invoke<SteamInstall | null>("detect_steam"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  isSteamRunning: () => invoke<boolean>("is_steam_running"),

  loadLibrary: (accountId3?: string, forceReload?: boolean) =>
    invoke<Game[]>("load_library", { accountId3: accountId3 ?? null, forceReload: forceReload ?? null }),
  loadCollections: (accountId3?: string) =>
    invoke<Collection[]>("load_collections", { accountId3: accountId3 ?? null }),

  updateCollection: (args: {
    collectionId: string;
    name?: string;
    addAppIds?: number[];
    removeAppIds?: number[];
    accountId3?: string;
  }) => invoke<Collection>("update_collection", args),
  createCollection: (name: string, accountId3?: string) =>
    invoke<Collection>("create_collection", { name, accountId3: accountId3 ?? null }),
  deleteCollection: (collectionId: string, accountId3?: string) =>
    invoke<void>("delete_collection", { collectionId, accountId3: accountId3 ?? null }),

  getFilterPresets: () => invoke<FilterPreset[]>("get_filter_presets"),
  saveFilterPreset: (preset: FilterPreset) =>
    invoke<FilterPreset>("save_filter_preset", { preset }),
  deleteFilterPreset: (id: string) => invoke<void>("delete_filter_preset", { id }),
  evaluateFilter: (filter: Filter, accountId3?: string) =>
    invoke<number[]>("evaluate_filter", { filter, accountId3: accountId3 ?? null }),
  saveSmartCollection: (name: string, filter: Filter, accountId3?: string) =>
    invoke<Collection>("save_smart_collection", { name, filter, accountId3: accountId3 ?? null }),
  refreshSmartCollection: (collectionId: string, accountId3?: string) =>
    invoke<Collection>("refresh_smart_collection", {
      collectionId,
      accountId3: accountId3 ?? null,
    }),

  fetchSteamStoreMetadata: (appIds: number[]) =>
    invoke<AppMeta[]>("fetch_steam_store_metadata", { appIds }),
  fetchOwnedGames: () => invoke<OwnedGame[]>("fetch_owned_games"),

  steamStoreSearch: (term: string) =>
    invoke<SteamSearchResult[]>("steam_store_search", { term }),

  // EmulationStation & ROM Management
  scaffoldEmulationFolders: (romsDir: string) =>
    invoke<ScaffoldReport>("scaffold_emulation_folders", { romsDir }),
  scanEmulationRoms: (romsDir: string, normalizeTitles?: boolean) =>
    invoke<PlatformGame[]>("scan_emulation_roms", { romsDir, normalizeTitles: normalizeTitles ?? true }),
  normalizeRomTitle: (title: string) =>
    invoke<string>("normalize_rom_title_cmd", { title }),

  // Native Plugin Engine
  getInstalledPlugins: () => invoke<PluginInfo[]>("get_installed_plugins"),
  togglePlugin: (id: string, enabled: boolean) => invoke<void>("toggle_plugin", { id, enabled }),
  scanAllPlatforms: (includeOtherDrives?: boolean) =>
    invoke<PlatformScan>("scan_all_platforms", { includeOtherDrives: includeOtherDrives ?? false }),

  detectPlatforms: (includeOtherDrives?: boolean) =>
    invoke<PlatformScan>("detect_platforms", { includeOtherDrives: includeOtherDrives ?? false }),
  findPlatformFolders: (platform: string) =>
    invoke<string[]>("find_platform_folders", { platform }),
  importPlatformGames: (games: PlatformGame[], accountId3?: string) =>
    invoke<ImportResult>("import_platform_games", { games, accountId3: accountId3 ?? null }),
  pickGameExecutable: () => invoke<string | null>("pick_game_executable"),
  pickGameDirectory: () => invoke<string | null>("pick_game_directory"),
  pickGameArtwork: () => invoke<string | null>("pick_game_artwork"),
  setGamePlatform: (appId: number, platform: string | null) =>
    invoke<void>("set_game_platform", { appId, platform }),
  getOwnedLibrary: () => invoke<OwnedLibrary>("get_owned_library"),
  refreshOwnedGames: () => invoke<OwnedLibrary>("refresh_owned_games"),

  // Warm (or refresh) the store-metadata cache for the whole library.
  syncLibraryMetadata: (accountId3?: string, force?: boolean, autoArtwork?: boolean) =>
    invoke<MetadataSyncResult>("sync_library_metadata", {
      accountId3: accountId3 ?? null,
      force: force ?? false,
      autoArtwork: autoArtwork ?? false
    }),

  // Refresh metadata for a specific game on demand across all providers.
  refreshGameMetadata: (appId: number, force?: boolean, accountId3?: string) =>
    invoke<AppMeta | null>("refresh_game_metadata", {
      appId,
      force: force ?? true,
      accountId3: accountId3 ?? null
    }),

  // Storage report: inspect disk footprint across platform libraries (read-only).
  getStorageReport: (accountId3?: string) =>
    invoke<StorageReport>("get_storage_report", { accountId3: accountId3 ?? null }),

  // Universal Cache & Persistence
  getCacheStats: () => invoke<CacheStats>("get_cache_stats"),

  // Launcher actions
  launchGame: (appId: number, launchUri?: string | null, exe?: string | null, platform?: string | null) =>
    invoke<boolean>("launch_game", { appId, launchUri: launchUri ?? null, exe: exe ?? null, platform: platform ?? null }),
  installGame: (appId: number, installUri?: string | null) =>
    invoke<boolean>("install_game", { appId, installUri: installUri ?? null }),
  uninstallGame: (appId: number, uninstallUri?: string | null) =>
    invoke<boolean>("uninstall_game", { appId, uninstallUri: uninstallUri ?? null }),
  openGameFolder: (path: string) => invoke<boolean>("open_game_folder", { path }),
  getCachedGameMetadata: (appId: number, name?: string | null, exe?: string | null) =>
    invoke<AppMeta | null>("get_cached_game_metadata", { appId, name: name ?? null, exe: exe ?? null }),

  // Universal Cache & Artwork Management
  clearMetadataCache: (accountId3?: string, redownload?: boolean) =>
    invoke<CacheResetReport>("clear_metadata_cache", {
      accountId3: accountId3 ?? null,
      redownload: redownload ?? true,
    }),
  getCachedArtwork: (appId: number, slot: string) =>
    invoke<string | null>("get_cached_artwork", { appId, slot }),
  getCachedArtworkBatch: (appIds: number[], slot?: string) =>
    invoke<Record<number, string>>("get_cached_artwork_batch", {
      appIds,
      slot: slot ?? null,
    }),
  cacheArtworkImage: (appId: number, slot: string, url: string) =>
    invoke<string | null>("cache_artwork_image", { appId, slot, url }),

  // Launcher Installation & Download Detection
  getInstalledLaunchers: () => invoke<LauncherStatus[]>("get_installed_launchers"),
  checkBinaryInstalled: (binaries: string[], customPath?: string | null) =>
    invoke<string | null>("check_binary_installed", { binaries, customPath: customPath ?? null }),
  installWingetPackage: (packageId: string, location?: string | null) =>
    invoke<boolean>("install_winget_package", { packageId, location: location ?? null }),
  isWingetAvailable: () => invoke<boolean>("is_winget_available"),
  getManagedDirectories: () => invoke<ManagedDirectories>("get_managed_directories"),
  setManagedDirectories: (toolsDir?: string | null, launchersDir?: string | null) =>
    invoke<boolean>("set_managed_directories", { toolsDir: toolsDir ?? null, launchersDir: launchersDir ?? null }),
  openLaunchersFolder: () => invoke<boolean>("open_launchers_folder"),
  installLegendary: () => invoke<LauncherStatus>("install_legendary"),
  openExternalUrl: (url: string) => invoke<boolean>("open_external_url", { url }),
  openToolsFolder: () => invoke<boolean>("open_tools_folder"),
  getLegendaryStatus: () => invoke<LegendaryAuthStatus>("get_legendary_status"),
  legendaryLogin: (code?: string | null, importLauncher?: boolean) =>
    invoke<LegendaryAuthStatus>("legendary_login", {
      code: code ?? null,
      importLauncher: importLauncher ?? false,
    }),
  legendaryLogout: () => invoke<LegendaryAuthStatus>("legendary_logout"),
  setLegendaryPath: (path: string | null) =>
    invoke<LegendaryAuthStatus>("set_legendary_path", { path: path ?? null }),
  openEpicLoginWindow: () => invoke<boolean>("open_epic_login_window"),
  getEosOverlayStatus: () => invoke<EosOverlayStatus>("get_eos_overlay_status"),
  installEosOverlay: () => invoke<EosOverlayStatus>("install_eos_overlay"),
  getFreeGamePromotions: (platforms?: string[]) =>
    invoke<FreeGamePromotion[]>("get_free_game_promotions", {
      platforms: platforms ?? null,
    }),
  getStoreDeals: (platforms?: string[]) =>
    invoke<StoreDeal[]>("get_store_deals", {
      platforms: platforms ?? null,
    }),
  getFeaturedStoreItems: (platforms?: string[]) =>
    invoke<FeaturedStoreItem[]>("get_featured_store_items", {
      platforms: platforms ?? null,
    }),
  getLibraryPatchNotes: (games?: [string, string][]) =>
    invoke<PatchNote[]>("get_library_patch_notes", {
      games: games ?? null,
    }),

  // In-Memory Fast Cache & Serverless KV Persistence (Redis-compatible)
  kvGet: (key: string) => invoke<string | null>("kv_get", { key }),
  kvSet: (key: string, value: string, ttlSecs?: number | null) =>
    invoke<boolean>("kv_set", { key, value, ttlSecs: ttlSecs ?? null }),
  kvDelete: (key: string) => invoke<boolean>("kv_delete", { key }),
  kvHas: (key: string) => invoke<boolean>("kv_has", { key }),
  kvKeys: (pattern?: string | null) => invoke<string[]>("kv_keys", { pattern: pattern ?? null }),
  kvMget: (keys: string[]) => invoke<Record<string, string>>("kv_mget", { keys }),
  kvMset: (pairs: [string, string][], ttlSecs?: number | null) =>
    invoke<boolean>("kv_mset", { pairs, ttlSecs: ttlSecs ?? null }),
  kvClearExpired: () => invoke<number>("kv_clear_expired"),
  async kvGetJson<T>(key: string): Promise<T | null> {
    const raw = await invoke<string | null>("kv_get", { key });
    if (!raw) return null;
    try {
      return JSON.parse(raw) as T;
    } catch {
      return null;
    }
  },
  async kvSetJson<T>(key: string, value: T, ttlSecs?: number | null): Promise<boolean> {
    const raw = JSON.stringify(value);
    return invoke<boolean>("kv_set", { key, value: raw, ttlSecs: ttlSecs ?? null });
  },
  fetchExternalJson: <T = unknown>(url: string, headers?: Record<string, string> | null) =>
    invoke<T>("fetch_external_json", { url, headers: headers ?? null }),
  pickGameImageFile: () => invoke<string | null>("pick_game_artwork"),
};

/** Normalize an unknown thrown value into a readable message. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

export { redis } from "./redis";
