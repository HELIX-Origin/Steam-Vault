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
  | "emulator"
  | "other";

export interface SteamSearchResult {
  id: number;
  name: string;
  tinyImage: string | null;
}

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
  launchUri?: string | null;
  installUri?: string | null;
  uninstallUri?: string | null;
  platformId?: string | null;
}

export interface Collection {
  id: string;
  name: string;
  favorite: boolean;
  hidden: boolean;
  dynamic: boolean;
  added: number[];
  removed: number[];
  filterSpec?: any;
}

export interface FilterPreset {
  id: string;
  name: string;
  filter: Filter;
  createdAt?: number;
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
  heroImage?: string | null;
  logoImage?: string | null;
  iconImage?: string | null;
  userScore: number | null;
  fetchedAt: number | null;
  screenshots?: string[];
  movies?: string[];
  locales?: Record<string, { title?: string; shortDescription?: string | null; aliases?: string[] }>;
  providers?: Record<string, any>;
}

export interface MediaItem {
  type: "video" | "image";
  url: string;
  thumbnailUrl?: string;
  title?: string;
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
  enableSteamSync?: boolean;
  autoCats: AutoCat[];
  platformSources: PlatformSource[];
  emulatorConfigs?: EmulatorConfig[];
  legendaryPath?: string | null;
  enableFreeGameNotifications?: boolean;
}

/** User-configured emulator mapping (mirrors the Rust `EmulatorConfig`). */
export interface EmulatorConfig {
  id: string;
  name: string;
  executablePath: string;
  romDirectories: string[];
  defaultArgs?: string | null;
}

/** Standard ES-DE system definition (mirrors the Rust `EsDeSystemInfo`). */
export interface EsDeSystemInfo {
  folderName: string;
  systemName: string;
  consolePlatform: string;
  extensions: string[];
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
  console?: string | null;
  installed?: boolean;
  installUri?: string | null;
  uninstallUri?: string | null;
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

// --- Storage management -----------------------------------------------------

/** One registered Steam library folder with its size and drive capacity. */
export interface LibraryEntry {
  index: number;
  path: string;
  label: string | null;
  exists: boolean;
  isSteamRoot: boolean;
  gameCount: number;
  totalBytes: number;
  diskTotal: number | null;
  diskFree: number | null;
}

/** An installed Steam app or external game shortcut and the library folder that holds it. */
export interface GamePlacement {
  appId: number;
  name: string;
  libraryPath: string;
  installDir: string | null;
  sizeOnDisk: number;
  installed: boolean;
  folderExists: boolean;
  platform?: Platform | null;
}

/** A mounted disk, for the drive overview. */
export interface DiskInfo {
  name: string;
  mount: string;
  total: number;
  free: number;
  removable: boolean;
}

/** Everything the Storage view needs in one round-trip. */
export interface StorageReport {
  root: string;
  libraries: LibraryEntry[];
  games: GamePlacement[];
  disks: DiskInfo[];
}

/** A dry-run summary of what a move would do. */
export interface MovePreview {
  from: string;
  to: string;
  bytes: number;
  targetFree: number | null;
  /** Whether the destination has room for the content plus headroom. */
  fits: boolean;
  /** True when the move is on one volume and will therefore be instant. */
  sameVolume: boolean;
  warnings: string[];
}

/** The result of a completed move. */
export interface MoveOutcome {
  from: string;
  to: string;
  bytes: number;
  files: number;
  sameVolume: boolean;
  /** `libraryfolders.vdf` files that were updated. */
  vdfUpdated: string[];
}

/** A throttled `storage-progress` event emitted during a long copy. */
export interface StorageProgress {
  label: string;
  done: number;
  total: number;
  files: number;
}

// --- Custom artwork ---------------------------------------------------------

/** One of Steam's five custom-artwork slots. */
export type ArtworkSlotId = "gridPortrait" | "gridLandscape" | "hero" | "logo" | "icon";

/** The on-disk state of one artwork slot for a game. */
export interface ArtworkSlotState {
  slot: ArtworkSlotId;
  label: string;
  present: boolean;
  path: string | null;
  bytes: number;
  modified: number | null;
}

/** The artwork state for one game across all five slots. */
export interface GameArtwork {
  appId: number;
  name: string;
  slots: ArtworkSlotState[];
  /** How many of the five slots are currently filled. */
  applied: number;
}

/** The result of applying or removing a slot's artwork. */
export interface ArtworkOutcome {
  appId: number;
  slot: ArtworkSlotId;
  /** `"applied"` or `"removed"`. */
  action: string;
  path: string | null;
  bytes: number;
  source: string | null;
  /** Where the replaced image was archived, when there was one. */
  backup: string | null;
}

// --- Local game catalog database --------------------------------------------

export interface CatalogStats {
  totalGames: number;
  totalMetadataCached: number;
  totalCachedArtworks?: number;
}

export interface CacheResetReport {
  totalCatalogGames: number;
  totalMetadataRecords: number;
  message: string;
}

export interface LauncherStatus {
  id: string;
  name: string;
  installed: boolean;
  executablePath?: string | null;
  alternativeMethod?: string | null;
  downloadUrl: string;
  alternativeUrl?: string | null;
  alternativeLabel?: string | null;
}

export interface LegendaryAuthStatus {
  isInstalled: boolean;
  isLoggedIn: boolean;
  accountName?: string | null;
  accountId?: string | null;
  authUrl: string;
  directAuthUrl?: string;
  executablePath?: string | null;
}

export interface EosOverlayStatus {
  isInstalled: boolean;
  isEnabled: boolean;
  version?: string | null;
  path?: string | null;
}

export interface FreeGamePromotion {
  id: string;
  title: string;
  platform: string;
  description?: string | null;
  url: string;
  imageUrl?: string | null;
  startDate?: string | null;
  endDate?: string | null;
  isActive: boolean;
  worth?: string | null;
}

export interface StoreDeal {
  id: string;
  title: string;
  platform: string;
  url: string;
  imageUrl?: string | null;
  discountPercent: number;
  originalPrice?: string | null;
  finalPrice: string;
  endDate?: string | null;
}

export interface PatchNote {
  id: string;
  appId: string;
  gameName: string;
  platform: string;
  title: string;
  url: string;
  date: string;
  author?: string | null;
  summary?: string | null;
}

export interface DownloadItem {
  id: string;
  name: string;
  category: "tool" | "overlay" | "artwork" | "metadata" | "game";
  progress: number;
  speed?: string;
  status: "queued" | "downloading" | "completed" | "failed";
  error?: string;
}


