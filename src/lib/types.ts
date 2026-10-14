// Shared TypeScript models mirroring the Rust backend (`src-tauri/src/steam/models.rs`).

export type GameKind = "steam" | "shortcut";

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
  kind: "genre" | "developer" | "publisher" | "flag" | "tag" | "releaseyear" | "userscore";
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
}

export interface AutoCatAssignment {
  collection: string;
  appIds: number[];
}

export interface AutoCatPlan {
  assignments: AutoCatAssignment[];
  gamesWithoutMetadata: number[];
}
