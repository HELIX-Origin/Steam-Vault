import type {
  UniversalPlugin,
  TestConnectionResult,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";
import { normalizeGameTitle } from "../utility/title-normalizer";
import { findHandRolledGame, backupMetadataForTitle } from "./developers";

export interface HelixDbSearchEntry {
  slug: string;
  title: string;
  developers: string[];
  publishers?: string[];
  releaseYear?: number;
  genres?: string[];
  categories?: string[];
  ageRating?: string | null;
  userScore?: number | null;
  capsuleImage?: string;
  heroImage?: string;
  logoImage?: string | null;
  keys: string[];
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
  genres: string[];
  categories: string[];
  tags: string[];
  ageRating?: string | null;
  userScore?: number | null;
  shortDescription: string;
  detailedDescription: string;
  website?: string | null;
  capsuleImage: string;
  heroImage: string;
  logoImage?: string | null;
  headerImage?: string | null;
  screenshots: string[];
  movies?: Array<{ title: string; url: string; thumbnail?: string | null }>;
  soundtracks?: Array<{ title: string; url: string; artist?: string | null }>;
  verified: boolean;
}

// Support official custom domain, www alias, and GitHub Pages origin fallback
const HELIX_DOMAINS = [
  "https://helix-origin.club/helix-gaming-db",
  "https://www.helix-origin.club/helix-gaming-db",
  "https://helix-origin.github.io/helix-gaming-db",
];

let cachedSearchIndex: HelixDbSearchIndex | null = null;
let lastIndexFetchTime = 0;
const INDEX_CACHE_TTL = 1000 * 60 * 30; // 30 minutes

function normalizeKey(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]/g, "")
    .trim();
}

/**
 * Fetch a JSON resource trying primary domain first, then fallback domain.
 */
async function fetchWithFallback<T>(relativePath: string): Promise<T | null> {
  const path = relativePath.startsWith("/") ? relativePath : `/${relativePath}`;
  for (const domain of HELIX_DOMAINS) {
    try {
      const url = `${domain}${path}`;
      const data = await api.fetchExternalJson<T>(url);
      if (data) return data;
    } catch {
      // Try next domain
    }
  }
  return null;
}

/**
 * Fetches and caches the lightweight search index from HELIX Gaming DB.
 */
export async function getHelixDbSearchIndex(): Promise<HelixDbSearchIndex | null> {
  const now = Date.now();
  if (cachedSearchIndex && now - lastIndexFetchTime < INDEX_CACHE_TTL) {
    return cachedSearchIndex;
  }
  const index = await fetchWithFallback<HelixDbSearchIndex>("/api/v1/search/index.json");
  if (index && Array.isArray(index.entries)) {
    cachedSearchIndex = index;
    lastIndexFetchTime = now;
    return index;
  }
  return cachedSearchIndex;
}

/**
 * Queries HELIX Gaming DB REST API for game metadata with seamless offline fallback.
 */
export async function queryHelixGamingDb(
  query: { appId?: number | string; name?: string; platform?: string }
): Promise<PluginMetadataResult | null> {
  const cleanTitle = normalizeGameTitle(query.name || "");
  if (!cleanTitle) return null;

  const key = normalizeKey(cleanTitle);

  // 1. Try remote REST API via cached search index
  try {
    const index = await getHelixDbSearchIndex();
    if (index) {
      const match = index.entries.find(
        (e) =>
          normalizeKey(e.slug) === key ||
          normalizeKey(e.title) === key ||
          e.keys.includes(key)
      );

      if (match) {
        const game = await fetchWithFallback<HelixDbGamePayload>(`/api/v1/games/${match.slug}.json`);
        if (game) {
          return {
            name: game.title,
            shortDescription: game.shortDescription,
            detailedDescription: game.detailedDescription,
            releaseDate: game.releaseDate,
            releaseYear: game.releaseYear,
            developers: game.developers,
            publishers: game.publishers,
            genres: game.genres,
            categories: game.categories,
            tags: game.tags,
            ageRating: game.ageRating || undefined,
            rating: typeof game.userScore === "number" ? game.userScore : undefined,
            capsuleImage: game.capsuleImage,
            heroImage: game.heroImage,
            logoImage: game.logoImage || undefined,
            headerImage: game.headerImage || undefined,
            screenshots: game.screenshots,
            movies: game.movies?.map((m) => m.url),
            soundtracks: game.soundtracks?.map((s) => s.url),
          };
        }
      }
    }
  } catch {
    // Offline or network error; fall through to local catalog
  }

  // 2. Offline fallback to hand-rolled developer catalog
  const handRolled = findHandRolledGame(query.name || "");
  if (handRolled) {
    return backupMetadataForTitle(query.name || "");
  }

  return null;
}

/**
 * Queries HELIX Gaming DB for artwork assets.
 */
export async function queryHelixGamingDbArtwork(
  query: { appId?: number | string; name?: string; platform?: string }
): Promise<PluginArtworkResult | null> {
  const meta = await queryHelixGamingDb(query);
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
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description:
    "Authoritative open REST metadata API and gaming database for PC, indie, Patreon, and emulation titles.",
  homepage: "https://helix-origin.club/helix-gaming-db",
  features: ["indie_metadata", "patreon_titles", "verified_artwork", "offline_fallback"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "endpoint_url",
          label: "Custom API Endpoint",
          type: "text",
          default: "https://helix-origin.club/helix-gaming-db",
          description: "Base URL for the HELIX Gaming DB REST API (defaults to official domain with GitHub Pages fallback).",
          placeholder: "https://helix-origin.club/helix-gaming-db",
        },
      ],
    };
  },

  async testConnection(config?: Record<string, any>): Promise<TestConnectionResult> {
    try {
      const manifest = await fetchWithFallback<{ name: string; totalGames: number }>("/api/v1/manifest.json");
      if (manifest && manifest.name) {
        return {
          success: true,
          message: `Connected to ${manifest.name} (${manifest.totalGames} games indexed).`,
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
      message: "Using local verified catalog with fallback enabled.",
    };
  },

  async queryMetadata(
    query: { appId?: number | string; name?: string; platform?: string },
    config?: Record<string, any>
  ): Promise<PluginMetadataResult | null> {
    return queryHelixGamingDb(query);
  },

  async queryArtwork(
    query: { appId?: number | string; name?: string; platform?: string },
    config?: Record<string, any>
  ): Promise<PluginArtworkResult | null> {
    return queryHelixGamingDbArtwork(query);
  },
};
