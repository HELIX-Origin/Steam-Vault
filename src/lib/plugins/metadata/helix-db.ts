import type {
  UniversalPlugin,
  TestConnectionResult,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";
import { normalizeGameTitle } from "../utility/title-normalizer";
import { titlesMatchStrict, backupMetadataForTitle } from "./developers";

export interface HelixDbSearchEntry {
  slug: string;
  title: string;
  canonicalTitle?: string;
  developers: string[];
  publishers?: string[];
  releaseYear?: number;
  genres?: string[];
  categories?: string[];
  platforms?: string[];
  ageRating?: string | null;
  userScore?: number | null;
  capsuleImage?: string;
  heroImage?: string;
  logoImage?: string | null;
  headerImage?: string | null;
  shard?: string;
  repository?: string;
  sourceUrl?: string;
  isLocalized?: boolean;
  keys: string[];
  isAdult?: boolean;
}

export interface HelixDbSearchIndex {
  version: string;
  updatedAt: string;
  totalEntries: number;
  entries: HelixDbSearchEntry[];
}

export interface HelixDbGamePayload {
  slug: string;
  title: string;
  aliases: string[];
  developers: string[];
  publishers: string[];
  releaseDate: string;
  releaseYear: number;
  platforms?: string[];
  genres: string[];
  categories: string[];
  tags: string[];
  ageRating?: string | null;
  userScore?: number | null;
  shortDescription: string;
  detailedDescription: string;
  website?: string | null;
  externalIds?: {
    steamAppId?: string | number | null;
    gogId?: string | number | null;
    itchId?: string | number | null;
    igdbId?: string | number | null;
  };
  capsuleImage: string;
  heroImage: string;
  logoImage?: string | null;
  headerImage?: string | null;
  iconImage?: string | null;
  screenshots: string[];
  movies?: Array<{ title: string; url: string; thumbnail?: string | null }>;
  soundtracks?: Array<{ title: string; url: string; artist?: string | null }>;
  verified: boolean;
  shard?: string;
  repository?: string;
  sourceUrl?: string;
  locales?: Record<
    string,
    {
      title?: string;
      shortDescription?: string;
      detailedDescription?: string;
      aliases?: string[];
    }
  >;
}

export interface HelixQueryOptions {
  customBase?: string;
  includeAdult?: boolean;
  tier?: "v2" | "v1";
  lang?: string;
}

// Support Cloudflare edge proxy (auto branch resolver), custom domain, www alias, and GitHub Pages origin fallback
const HELIX_DOMAINS = [
  "https://api.helix-origin.club/helix-gaming-db",
  "https://helix-origin.club/helix-gaming-db",
  "https://www.helix-origin.club/helix-gaming-db",
  "https://helix-origin.github.io/helix-gaming-db",
];

// Dual-tier in-memory index cache
let cachedV2Index: HelixDbSearchIndex | null = null;
let lastV2FetchTime = 0;

let cachedV3Index: HelixDbSearchIndex | null = null;
let lastV3FetchTime = 0;

// In-memory detailed game payload cache
const cachedGames = new Map<string, HelixDbGamePayload>();
const INDEX_CACHE_TTL = 1000 * 60 * 30; // 30 minutes

function normalizeKey(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "")
    .trim();
}

/**
 * Fetch a JSON resource trying candidate domains sequentially with graceful error handling.
 */
async function fetchWithFallback<T>(relativePath: string, customBase?: string): Promise<T | null> {
  const path = relativePath.startsWith("/") ? relativePath : `/${relativePath}`;
  const candidates = customBase ? [customBase, ...HELIX_DOMAINS] : HELIX_DOMAINS;
  for (const domain of candidates) {
    try {
      const cleanDomain = domain.endsWith("/") ? domain.slice(0, -1) : domain;
      const url = `${cleanDomain}${path}`;
      const data = await api.fetchExternalJson<T>(url);
      if (data) return data;
    } catch {
      // Try next candidate
    }
  }
  return null;
}

/**
 * Fetches and caches the search index from HELIX Gaming DB with dual-tier support (v2 relational, v3 adult)
 * and automatic offline fallback via persistent serverless KV storage.
 */
export async function getHelixDbSearchIndex(
  tier: "v2" | "v3" = "v2",
  customBase?: string
): Promise<HelixDbSearchIndex | null> {
  const now = Date.now();
  const isV3 = tier === "v3";
  const cached = isV3 ? cachedV3Index : cachedV2Index;
  const lastTime = isV3 ? lastV3FetchTime : lastV2FetchTime;

  if (cached && now - lastTime < INDEX_CACHE_TTL) {
    return cached;
  }

  const path = isV3 ? "/api/v3/search/index.json" : "/api/v2/search/index.json";
  const kvKey = `helix_db_index_${tier}`;

  // 1. Try remote fetch
  try {
    const index = await fetchWithFallback<HelixDbSearchIndex>(path, customBase);
    if (index && Array.isArray(index.entries)) {
      if (isV3) {
        cachedV3Index = index;
        lastV3FetchTime = now;
      } else {
        cachedV2Index = index;
        lastV2FetchTime = now;
      }
      // Persist to serverless KV cache asynchronously
      try {
        await api.kvSetJson(kvKey, index, 86400 * 7); // 7-day retention
      } catch {
        // Non-fatal
      }
      return index;
    }
  } catch {
    // Network offline; proceed to persistent cache
  }

  // 2. Offline fallback: read persistent KV cache
  try {
    const persisted = await api.kvGetJson<HelixDbSearchIndex>(kvKey);
    if (persisted && Array.isArray(persisted.entries)) {
      if (isV3) {
        cachedV3Index = persisted;
      } else {
        cachedV2Index = persisted;
      }
      return persisted;
    }
  } catch {
    // Non-fatal
  }

  return cached;
}

/**
 * Fetches and caches a detailed game payload from HELIX Gaming DB with federated multi-repo resolution.
 */
export async function getHelixDbGame(
  slug: string,
  tier: "v2" | "v3" = "v2",
  customBase?: string,
  meta?: { shard?: string; repository?: string; sourceUrl?: string }
): Promise<HelixDbGamePayload | null> {
  const memKey = `${tier}:${slug}`;
  if (cachedGames.has(memKey)) {
    return cachedGames.get(memKey)!;
  }

  const kvKey = `helix_db_game_${tier}_${slug}`;

  // 1. Direct source URL if provided by federated shard index
  if (meta?.sourceUrl) {
    try {
      const data = await api.fetchExternalJson<HelixDbGamePayload>(meta.sourceUrl);
      if (data && data.title) {
        cachedGames.set(memKey, data);
        try {
          await api.kvSetJson(kvKey, data, 86400 * 14);
        } catch {
          // Non-fatal
        }
        return data;
      }
    } catch {
      // Fall through to shard repository or primary registry
    }
  }

  // 2. Federated multi-repo resolution if hosted on dedicated shard repository
  if (meta?.repository && meta.repository !== "HELIX-Origin/helix-gaming-db") {
    const rawShardUrl = `https://raw.githubusercontent.com/${meta.repository}/main/api/${tier}/games/${slug}.json`;
    try {
      const data = await api.fetchExternalJson<HelixDbGamePayload>(rawShardUrl);
      if (data && data.title) {
        cachedGames.set(memKey, data);
        try {
          await api.kvSetJson(kvKey, data, 86400 * 14);
        } catch {
          // Non-fatal
        }
        return data;
      }
    } catch {
      // Fall through to primary registry
    }
  }

  // 3. Try primary registry / CDN domains
  try {
    const path = tier === "v3" ? `/api/v3/games/${slug}.json` : `/api/v2/games/${slug}.json`;
    const game = await fetchWithFallback<HelixDbGamePayload>(path, customBase);
    if (game && game.title) {
      cachedGames.set(memKey, game);
      try {
        await api.kvSetJson(kvKey, game, 86400 * 14); // 14-day retention
      } catch {
        // Non-fatal
      }
      return game;
    }
  } catch {
    // Network offline; proceed to persistent cache
  }

  // 4. Offline fallback: read persistent KV cache
  try {
    const persisted = await api.kvGetJson<HelixDbGamePayload>(kvKey);
    if (persisted && persisted.title) {
      cachedGames.set(memKey, persisted);
      return persisted;
    }
  } catch {
    // Non-fatal
  }

  return null;
}

function mapHelixPayloadToResult(game: HelixDbGamePayload, lang?: string): PluginMetadataResult {
  const loc = lang && game.locales && game.locales[lang] ? game.locales[lang] : null;

  return {
    name: loc?.title || game.title,
    shortDescription: loc?.shortDescription || game.shortDescription,
    detailedDescription: loc?.detailedDescription || game.detailedDescription,
    releaseDate: game.releaseDate,
    releaseYear: game.releaseYear,
    developers: game.developers,
    publishers: game.publishers,
    genres: game.genres,
    categories: game.categories,
    tags: game.tags,
    ageRating: game.ageRating || undefined,
    rating: typeof game.userScore === "number" ? game.userScore : undefined,
    userScore: typeof game.userScore === "number" ? game.userScore : undefined,
    capsuleImage: game.capsuleImage,
    heroImage: game.heroImage,
    logoImage: game.logoImage || undefined,
    headerImage: game.headerImage || undefined,
    iconImage: game.iconImage || undefined,
    screenshots: game.screenshots,
    movies: game.movies?.map((m) => m.url),
    soundtracks: game.soundtracks?.map((s) => s.url),
  };
}

function mapHelixIndexEntryToResult(entry: HelixDbSearchEntry): PluginMetadataResult {
  return {
    name: entry.title,
    releaseYear: entry.releaseYear,
    developers: entry.developers,
    publishers: entry.publishers,
    genres: entry.genres,
    categories: entry.categories,
    ageRating: entry.ageRating || undefined,
    rating: typeof entry.userScore === "number" ? entry.userScore : undefined,
    userScore: typeof entry.userScore === "number" ? entry.userScore : undefined,
    capsuleImage: entry.capsuleImage,
    heroImage: entry.heroImage,
    logoImage: entry.logoImage || undefined,
    headerImage: entry.headerImage || undefined,
  };
}

/**
 * Queries HELIX Gaming DB Three-Tier REST API for game metadata with multi-tier search
 * and seamless offline fallback.
 */
export async function queryHelixGamingDb(
  query: { appId?: number | string; name?: string; platform?: string },
  options?: HelixQueryOptions
): Promise<PluginMetadataResult | null> {
  const cleanTitle = normalizeGameTitle(query.name || "");
  if (!cleanTitle && !query.appId) return null;

  const key = cleanTitle ? normalizeKey(cleanTitle) : "";
  const includeAdult = options?.includeAdult ?? true;

  // Search tiers in sequence: v2 (main catalog), then v3 (adult catalog if enabled)
  const searchTiers: Array<"v2" | "v3"> = includeAdult ? ["v2", "v3"] : ["v2"];

  for (const tier of searchTiers) {
    const index = await getHelixDbSearchIndex(tier, options?.customBase);
    if (!index || !Array.isArray(index.entries)) continue;

    const match = index.entries.find((e) => {
      // 1. Direct key match from index keys
      if (key && e.keys.includes(key)) return true;

      // 2. Slug / title match
      const eSlugKey = normalizeKey(e.slug);
      const eTitleKey = normalizeKey(e.title);
      if (key && (eSlugKey === key || eTitleKey === key)) return true;

      // 3. Strict title match (handles edition suffixes, sequels, subtitles)
      if (cleanTitle && titlesMatchStrict(cleanTitle, e.title)) return true;

      return false;
    });

    if (match) {
      // Fetch full game payload (with federated multi-repo resolution)
      const game = await getHelixDbGame(match.slug, tier, options?.customBase, {
        shard: match.shard,
        repository: match.repository,
        sourceUrl: match.sourceUrl,
      });
      if (game) {
        return mapHelixPayloadToResult(game, options?.lang);
      }

      // Offline fallback: synthesize result from rich search index entry
      return mapHelixIndexEntryToResult(match);
    }
  }

  // Fallback: studio-level credits and genres from known developer franchise detection
  if (cleanTitle) {
    return backupMetadataForTitle(cleanTitle);
  }

  return null;
}

/**
 * Queries HELIX Gaming DB for high-resolution artwork assets.
 */
export async function queryHelixGamingDbArtwork(
  query: { appId?: number | string; name?: string; platform?: string },
  options?: HelixQueryOptions
): Promise<PluginArtworkResult | null> {
  const meta = await queryHelixGamingDb(query, options);
  if (!meta) return null;

  return {
    capsuleImage: meta.capsuleImage,
    heroImage: meta.heroImage,
    logoImage: meta.logoImage,
    headerImage: meta.headerImage,
    screenshots: meta.screenshots,
  };
}

export const helixDbMetadataPlugin: UniversalPlugin = {
  id: "metadata-helix",
  name: "HELIX Gaming DB",
  version: "1.1.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description:
    "Authoritative open REST metadata API and gaming database for PC, indie, Patreon, and emulation titles.",
  homepage: "https://helix-origin.club/helix-gaming-db",
  features: ["indie_metadata", "patreon_titles", "verified_artwork", "offline_fallback", "adult_v3_tier", "i18n_localization", "multi_repo_federation"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "endpoint_url",
          label: "Custom API Endpoint",
          type: "text",
          default: "https://api.helix-origin.club/helix-gaming-db",
          description: "Base URL for the HELIX Gaming DB REST API (defaults to Cloudflare edge proxy with GitHub Pages fallback).",
          placeholder: "https://api.helix-origin.club/helix-gaming-db",
        },
        {
          key: "preferred_language",
          label: "Metadata Language",
          type: "select",
          default: "en",
          options: [
            { value: "en", label: "English" },
            { value: "ja", label: "日本語 (Japanese)" },
            { value: "zh-CN", label: "简体中文 (Simplified Chinese)" },
            { value: "zh-TW", label: "繁體中文 (Traditional Chinese)" },
            { value: "es", label: "Español (Spanish)" },
            { value: "fr", label: "Français (French)" },
            { value: "de", label: "Deutsch (German)" },
            { value: "it", label: "Italiano (Italian)" },
            { value: "pt-BR", label: "Português (Brasil)" },
            { value: "ru", label: "Русский (Russian)" },
            { value: "ko", label: "한국어 (Korean)" },
            { value: "pl", label: "Polski (Polish)" },
          ],
          description: "Language to use for localized game titles, synopses, and taxonomy when available.",
        },
        {
          key: "include_adult_content",
          label: "Include Adult / 18+ Games (v3 Tier)",
          type: "boolean",
          default: true,
          description: "Search the dedicated v3 Adult (18+) tier for uncensored indie and Patreon titles.",
        },
        {
          key: "api_tier",
          label: "Preferred API Version",
          type: "select",
          default: "v2",
          options: [
            { value: "v2", label: "v2 Relational Discovery (Recommended)" },
            { value: "v1", label: "v1 Client Tier (Legacy)" },
          ],
          description: "API version tier to prioritize for game metadata and search.",
        },
      ],
    };
  },

  async testConnection(config?: Record<string, any>): Promise<TestConnectionResult> {
    try {
      const manifest = await fetchWithFallback<{
        name: string;
        version?: string;
        supportedVersions?: string[];
        totalGames?: number;
      }>("/api/manifest.json", config?.endpoint_url);

      if (manifest && manifest.name) {
        const versions = manifest.supportedVersions?.join(", ") || "v1, v2, v3";
        return {
          success: true,
          message: `Connected to ${manifest.name} (Tiers: ${versions}).`,
        };
      }
    } catch (e: any) {
      return {
        success: false,
        message: `Connection failed: ${e?.message || e}`,
      };
    }
    return {
      success: true,
      message: "Using local verified cache with offline fallback enabled.",
    };
  },

  async queryMetadata(
    query: { appId?: number | string; name?: string; platform?: string },
    config?: Record<string, any>
  ): Promise<PluginMetadataResult | null> {
    return queryHelixGamingDb(query, {
      customBase: config?.endpoint_url,
      includeAdult: config?.include_adult_content !== false,
      tier: config?.api_tier === "v1" ? "v1" : "v2",
      lang: config?.preferred_language || "en",
    });
  },

  async queryArtwork(
    query: { appId?: number | string; name?: string; platform?: string },
    config?: Record<string, any>
  ): Promise<PluginArtworkResult | null> {
    return queryHelixGamingDbArtwork(query, {
      customBase: config?.endpoint_url,
      includeAdult: config?.include_adult_content !== false,
      tier: config?.api_tier === "v1" ? "v1" : "v2",
    });
  },
};
