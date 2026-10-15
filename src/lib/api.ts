// Typed wrappers around the Tauri IPC commands.
// Tauri maps camelCase JS arguments to snake_case Rust parameters automatically.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppMeta,
  ApplyResult,
  ArtworkOutcome,
  ArtworkSlotId,
  AutoCatPlan,
  CatalogStats,
  CategorizationPlan,
  CategorizeRequest,
  Collection,
  Filter,
  FilterPreset,
  Game,
  GameArtwork,
  IgdbGame,
  ImportResult,
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

  // Warm the store-metadata cache for the whole library.
  syncLibraryMetadata: (accountId3?: string) =>
    invoke<MetadataSyncResult>("sync_library_metadata", { accountId3: accountId3 ?? null }),

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
    kind: "grids" | "heroes" | "logos" | "icons",
  ) => invoke<SgdbAsset[]>("steamgriddb_for_game", { appId, name, kind }),
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
};

/** Normalize an unknown thrown value into a readable message. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}
