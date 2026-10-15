// Shared TypeScript models mirroring the Rust backend (`src-tauri/src/steam/models.rs`).

export type GameKind = "steam" | "shortcut";

/** A storefront / launcher a game can come from. */
export type Platform =
  | "steam"
  | "gog"
  | "epic"
  | "itch"
  | "ubisoft"
  | "ea"
  | "battlenet"
  | "amazon"
  | "xbox"
  | "rockstar"
  | "other";

export interface SteamLibrary {
  index: number;
  path: string;
  label: string | null;
  appIds: number[];
}

export interface SteamAccount {
  id3: string;
  id64: string | null;
  accountName: string | null;
  personaName: string | null;
}

export interface SteamInstall {
  root: string;
  libraries: SteamLibrary[];
  accounts: SteamAccount[];
}

export interface Game {
  appId: number;
  name: string;
  kind: GameKind;
  libraryPath: string | null;
  installDir: string | null;
  sizeOnDisk: number;
  installed: boolean;
  lastPlayed: number | null;
  playtimeMinutes: number | null;
  collections: string[];
  favorite: boolean;
  hidden: boolean;
  /** Steam for real apps; the source storefront for imported shortcuts. */
  platform: Platform;
  shortcutId: number | null;
  exe: string | null;
  startDir: string | null;
  icon: string | null;
}

export interface Collection {
  id: string;
  name: string;
  favorite: boolean;
  hidden: boolean;
  dynamic: boolean;
  added: number[];
  removed: number[];
}

export interface AppMeta {
  appId: number;
  name: string | null;
  developers: string[];
  publishers: string[];
  genres: string[];
  categories: string[];
  tags: string[];
  releaseDate: string | null;
  releaseYear: number | null;
  shortDescription: string | null;
  headerImage: string | null;
  capsuleImage: string | null;
  userScore: number | null;
  fetchedAt: number | null;
}

export interface ProviderInfo {
  id: string;
  label: string;
  requiresKey: boolean;
  category: string;
  description: string;
}

export interface OwnedGame {
  appid: number;
  name: string | null;
  playtime_forever: number | null;
  playtime_2weeks: number | null;
  img_icon_url: string | null;
}

export interface SgdbGame {
  id: number;
  name: string;
  release_date: number | null;
}

export interface SgdbAsset {
  id: number;
  url: string;
  thumb: string | null;
  width: number | null;
  height: number | null;
  style: string | null;
  mime: string | null;
}

export interface IgdbNamed {
  name: string;
}

export interface IgdbCompany {
  company: IgdbNamed | null;
  developer: boolean | null;
  publisher: boolean | null;
}

export interface IgdbGame {
  id: number;
  name: string;
  summary: string | null;
  first_release_date: number | null;
  genres: IgdbNamed[];
  involved_companies: IgdbCompany[];
  cover: { url: string | null } | null;
}

export interface AutoCat {
  id: string;
  name: string;
  kind:
    | "genre"
    | "developer"
    | "publisher"
    | "flag"
    | "tag"
    | "releaseyear"
    | "userscore"
    | "platform"
    | "installedstate"
    | "playtime"
    | "name";
  enabled: boolean;
  prefix: string | null;
  maxCategories: number | null;
  removeExisting: boolean;
  include: string[];
  exclude: string[];
}

export interface Settings {
  steamRoot: string | null;
  accountId3: string | null;
  steamApiKey: string | null;
  steamgriddbApiKey: string | null;
  igdbClientId: string | null;
  igdbClientSecret: string | null;
  countryCode: string;
  language: string;
  autoBackup: boolean;
  autoCats: AutoCat[];
  platformSources: PlatformSource[];
}

/** Per-platform detection preference (mirrors the Rust `PlatformSource`). */
export interface PlatformSource {
  /** Stable platform id: steam, gog, epic, itch, ubisoft, ea, battlenet, … */
  id: string;
  enabled: boolean;
  /** Extra folders to scan for this platform (for example an external drive). */
  roots: string[];
}

export interface AutoCatAssignment {
  collection: string;
  appIds: number[];
}

export interface AutoCatPlan {
  assignments: AutoCatAssignment[];
  gamesWithoutMetadata: number[];
}

// --- External platforms -----------------------------------------------------

/** A game discovered by scanning a non-Steam launcher. */
export interface PlatformGame {
  name: string;
  platform: Platform;
  platformId: string | null;
  installDir: string | null;
  exe: string | null;
  sizeOnDisk: number | null;
  launchUri: string | null;
  source: string | null;
}

/** Per-platform scan summary (availability, count, sources consulted). */
export interface PlatformStatus {
  platform: Platform;
  label: string;
  available: boolean;
  count: number;
  roots: string[];
  notes: string | null;
}

export interface PlatformScan {
  games: PlatformGame[];
  platforms: PlatformStatus[];
}

export interface ImportResult {
  added: Game[];
  skipped: string[];
}

// --- Full owned Steam library ----------------------------------------------

export interface OwnedGameRecord {
  appId: number;
  name: string;
  playtimeMinutes: number;
  playtimeTwoWeeks: number;
  iconUrl: string | null;
}

export interface OwnedLibrary {
  fetchedAt: number | null;
  steamId64: string | null;
  games: OwnedGameRecord[];
}

// --- Create collections from a sorting option ------------------------------

export type CategorizeKind = AutoCat["kind"];

export interface Filter {
  allowed: string[];
  required: string[];
  excluded: string[];
  search: string | null;
  kind: GameKind | null;
}

export interface CategorizeRequest {
  kind: CategorizeKind;
  prefix?: string | null;
  maxCategories?: number | null;
  include?: string[];
  exclude?: string[];
  removeExisting?: boolean;
  onlyInstalled?: boolean;
  /** Download missing store metadata before computing (default true). */
  fetchMetadata?: boolean;
  filter?: Filter | null;
}

/** Outcome of warming the local metadata cache. */
export interface MetadataSyncResult {
  requested: number;
  fetched: number;
  tagsFetched: number;
  cached: number;
  unavailable: number;
}

export interface CategorizationPlan {
  kind: CategorizeKind;
  assignments: AutoCatAssignment[];
  categorizedGames: number;
  totalGames: number;
  gamesWithoutMetadata: number[];
  removeExisting: boolean;
}

export interface ApplyResult {
  collectionsCreated: number;
  collectionsUpdated: number;
  gamesAssigned: number;
}
