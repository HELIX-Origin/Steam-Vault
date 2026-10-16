// Typed wrappers around the Tauri IPC commands.
// Tauri maps camelCase JS arguments to snake_case Rust parameters automatically.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppMeta,
  ApplyResult,
  ArtworkOutcome,
  ArtworkSlotId,
  AutoCatPlan,
  CacheResetReport,
  CatalogStats,
  CategorizationPlan,
  CategorizeRequest,
  Collection,
  EsDeSystemInfo,
  Filter,
  FilterPreset,
  Game,
  GameArtwork,
  IgdbGame,
  ImportResult,
  LauncherStatus,
  LegendaryAuthStatus,
  MetadataSyncResult,
  MoveOutcome,
  MovePreview,
  OwnedGame,
  OwnedLibrary,
  PlatformGame,
  PlatformScan,
  ProviderInfo,
  Settings,
  SgdbAsset,
  SgdbGame,
  SteamInstall,
  SteamSearchResult,
  StorageReport,
} from "./types";

export const api = {
  getProviders: () => invoke<ProviderInfo[]>("get_providers"),
  detectSteam: () => invoke<SteamInstall | null>("detect_steam"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  isSteamRunning: () => invoke<boolean>("is_steam_running"),

  loadLibrary: (accountId3?: string) =>
    invoke<Game[]>("load_library", { accountId3: accountId3 ?? null }),
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

  steamGridDbSearch: (term: string) => invoke<SgdbGame[]>("steamgriddb_search", { term }),
  steamGridDbAssets: (gameId: number, kind: "grids" | "heroes" | "logos" | "icons") =>
    invoke<SgdbAsset[]>("steamgriddb_assets", { gameId, kind }),

  igdbSearch: (query: string) => invoke<IgdbGame[]>("igdb_search", { query }),
  pcgamingwikiLookup: (name: string) => invoke<AppMeta | null>("pcgamingwiki_lookup", { name }),
  wikipediaSummary: (title: string) => invoke<string | null>("wikipedia_summary", { title }),
  wikipediaGameLookup: (title: string) =>
    invoke<AppMeta | null>("wikipedia_game_lookup", { title }),
  vndbLookup: (title: string) => invoke<AppMeta | null>("vndb_lookup", { title }),
  steamStoreSearch: (term: string) =>
    invoke<SteamSearchResult[]>("steam_store_search", { term }),

  computeAutocats: (accountId3?: string) =>
    invoke<AutoCatPlan>("compute_autocats", { accountId3: accountId3 ?? null }),

  detectPlatforms: (includeOtherDrives?: boolean) =>
    invoke<PlatformScan>("detect_platforms", { includeOtherDrives: includeOtherDrives ?? false }),
  findPlatformFolders: (platform: string) =>
    invoke<string[]>("find_platform_folders", { platform }),
  importPlatformGames: (games: PlatformGame[], accountId3?: string) =>
    invoke<ImportResult>("import_platform_games", { games, accountId3: accountId3 ?? null }),
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

  // Create collections from a chosen sorting option.
  computeCategorization: (request: CategorizeRequest, accountId3?: string) =>
    invoke<CategorizationPlan>("compute_categorization", {
      request,
      accountId3: accountId3 ?? null,
    }),
  applyCategorization: (plan: CategorizationPlan, accountId3?: string) =>
    invoke<ApplyResult>("apply_categorization", { plan, accountId3: accountId3 ?? null }),

  // Storage management: inspect, register, and relocate Steam libraries.
  getStorageReport: (accountId3?: string) =>
    invoke<StorageReport>("get_storage_report", { accountId3: accountId3 ?? null }),
  previewMoveLibrary: (from: string, to: string) =>
    invoke<MovePreview>("preview_move_library", { from, to }),
  moveLibrary: (from: string, to: string) =>
    invoke<MoveOutcome>("move_library", { from, to }),
  previewMoveGame: (appId: number, target: string) =>
    invoke<MovePreview>("preview_move_game", { appId, target }),
  moveGame: (appId: number, target: string) =>
    invoke<MoveOutcome>("move_game", { appId, target }),
  addLibraryFolder: (path: string) => invoke<string[]>("add_library_folder", { path }),
  removeLibraryFolder: (path: string) => invoke<string[]>("remove_library_folder", { path }),

  // Custom artwork: inspect, preview, apply, and remove grid/hero/logo/icon art.
  getArtwork: (accountId3?: string, appIds?: number[]) =>
    invoke<GameArtwork[]>("get_artwork", { accountId3: accountId3 ?? null, appIds: appIds ?? null }),
  readArtwork: (appId: number, slot: ArtworkSlotId, accountId3?: string) =>
    invoke<string | null>("read_artwork", { appId, slot, accountId3: accountId3 ?? null }),
  applyArtworkFromUrl: (appId: number, slot: ArtworkSlotId, url: string, accountId3?: string) =>
    invoke<ArtworkOutcome>("apply_artwork_from_url", {
      appId,
      slot,
      url,
      accountId3: accountId3 ?? null,
    }),
  applyArtworkFromFile: (appId: number, slot: ArtworkSlotId, path: string, accountId3?: string) =>
    invoke<ArtworkOutcome>("apply_artwork_from_file", {
      appId,
      slot,
      path,
      accountId3: accountId3 ?? null,
    }),
  removeArtwork: (appId: number, slot: ArtworkSlotId, accountId3?: string) =>
    invoke<ArtworkOutcome>("remove_artwork", { appId, slot, accountId3: accountId3 ?? null }),
  steamGridDbForGame: (
    appId: number,
    name: string,
    kind: "grids" | "grids_portrait" | "grids_landscape" | "heroes" | "logos" | "icons",
    nsfw?: boolean,
  ) => invoke<SgdbAsset[]>("steamgriddb_for_game", { appId, name, kind, nsfw: nsfw ?? false }),
  autoFetchArtworkForGames: (appIds?: number[], accountId3?: string) =>
    invoke<number>("auto_fetch_artwork_for_games", {
      appIds: appIds ?? null,
      accountId3: accountId3 ?? null,
    }),

  // Local game catalog & compressed archives
  getCatalogStats: () => invoke<CatalogStats>("get_catalog_stats"),
  exportCatalogArchive: (path: string) => invoke<number>("export_catalog_archive", { path }),
  importCatalogArchive: (path: string) => invoke<number>("import_catalog_archive", { path }),
  syncOnlineCatalog: (customUrl?: string) =>
    invoke<number>("sync_online_catalog", { customUrl: customUrl ?? null }),
  fetchOnlineGameEntry: (titleOrSlug: string) =>
    invoke<AppMeta | null>("fetch_online_game_entry", { titleOrSlug }),

  // Launcher actions
  launchGame: (appId: number, launchUri?: string | null, exe?: string | null) =>
    invoke<boolean>("launch_game", { appId, launchUri: launchUri ?? null, exe: exe ?? null }),
  installGame: (appId: number, installUri?: string | null) =>
    invoke<boolean>("install_game", { appId, installUri: installUri ?? null }),
  uninstallGame: (appId: number, uninstallUri?: string | null) =>
    invoke<boolean>("uninstall_game", { appId, uninstallUri: uninstallUri ?? null }),
  openGameFolder: (path: string) => invoke<boolean>("open_game_folder", { path }),

  // EmulationStation (ES-DE) Integration & ROM Directory Scaffolding
  getEsDeSystems: () => invoke<EsDeSystemInfo[]>("get_es_de_systems"),
  scaffoldEsDeDirectories: (targetPath: string) =>
    invoke<string[]>("scaffold_es_de_directories", { targetPath }),
  searchCatalog: (term: string) => invoke<AppMeta[]>("search_catalog", { term }),

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
};

/** Normalize an unknown thrown value into a readable message. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

export { redis } from "./redis";
