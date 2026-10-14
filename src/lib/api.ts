// Typed wrappers around the Tauri IPC commands.
// Tauri maps camelCase JS arguments to snake_case Rust parameters automatically.

import { invoke } from "@tauri-apps/api/core";
import type {
  AppMeta,
  AutoCatPlan,
  Collection,
  Game,
  IgdbGame,
  OwnedGame,
  ProviderInfo,
  Settings,
  SgdbAsset,
  SgdbGame,
  SteamInstall,
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

  fetchSteamStoreMetadata: (appIds: number[]) =>
    invoke<AppMeta[]>("fetch_steam_store_metadata", { appIds }),
  fetchOwnedGames: () => invoke<OwnedGame[]>("fetch_owned_games"),

  steamGridDbSearch: (term: string) => invoke<SgdbGame[]>("steamgriddb_search", { term }),
  steamGridDbAssets: (gameId: number, kind: "grids" | "heroes" | "logos" | "icons") =>
    invoke<SgdbAsset[]>("steamgriddb_assets", { gameId, kind }),

  igdbSearch: (query: string) => invoke<IgdbGame[]>("igdb_search", { query }),
  pcgamingwikiLookup: (name: string) => invoke<AppMeta | null>("pcgamingwiki_lookup", { name }),
  wikipediaSummary: (title: string) => invoke<string | null>("wikipedia_summary", { title }),

  computeAutocats: (accountId3?: string) =>
    invoke<AutoCatPlan>("compute_autocats", { accountId3: accountId3 ?? null }),
};

/** Normalize an unknown thrown value into a readable message. */
export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}
