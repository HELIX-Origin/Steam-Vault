import type {
  UniversalPlugin,
  TestConnectionResult,
  PluginArtworkResult,
  PluginMetadataResult,
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";
import { api } from "$lib/api";
import { normalizeGameTitle } from "../utility/title-normalizer";
import {
  titlesMatchStrict,
  validateMetadataCandidate,
  backupMetadataForTitle,
} from "./developers";
import {
  queryHelixGamingDb,
  queryHelixGamingDbArtwork,
} from "./helix-db";

/**
 * Resolves a Steam AppID from either a native numeric AppID (< 50,000,000) or a title search via the Rust backend.
 */
export async function resolveSteamAppId(appId?: number | string, name?: string): Promise<number | null> {
  const numId = Number(appId);
  if (numId > 0 && numId < 50_000_000) {
    return numId;
  }
  const cleanName = normalizeGameTitle(name || "");
  if (!cleanName) return null;
  try {
    const results = await api.steamStoreSearch(cleanName);
    if (results && results.length > 0) {
      const match = results.find(
        (r) => titlesMatchStrict(cleanName, r.name) && validateMetadataCandidate(cleanName, r.name)
      );
      if (match) return match.id;
    }
    const base = cleanName.split(/\s+[-:]\s+/)[0]?.trim();
    if (base && base !== cleanName) {
      const baseResults = await api.steamStoreSearch(base);
      if (baseResults && baseResults.length > 0) {
        const match = baseResults.find(
          (r) => titlesMatchStrict(cleanName, r.name) && validateMetadataCandidate(cleanName, r.name)
        );
        if (match) return match.id;
      }
    }
  } catch {
    // Ignore search failure
  }
  return null;
}

/**
 * Global helper querying official Steam Storefront API via Rust HTTP client (`fetchExternalJson`).
 */
export async function querySteamStoreFront(
  query: { appId?: number | string; name?: string; platform?: string },
  config?: Record<string, any>
): Promise<PluginMetadataResult | null> {
  // Avoid matching console-exclusive ROMs (e.g. GameCube Zelda) to unrelated Steam titles
  const plat = (query.platform || "").toLowerCase();
  const isConsoleRom =
    plat &&
    ![
      "steam",
      "gog",
      "epic",
      "itch",
      "xbox",
      "ea",
      "ubisoft",
      "battlenet",
      "amazon",
      "rockstar",
      "standalone",
      "other",
      "pc",
    ].includes(plat);
  if (isConsoleRom) return null;

  const resolvedId = await resolveSteamAppId(query.appId, query.name);
  if (!resolvedId) return null;

  const cc = (config?.countryCode || "us").trim();
  const lang = (config?.language || "english").trim();

  try {
    const data = await api.fetchExternalJson<Record<string, any>>(
      `https://store.steampowered.com/api/appdetails?appids=${resolvedId}&cc=${cc}&l=${lang}`
    );
    const entry = data?.[String(resolvedId)];
    if (!entry?.success || !entry?.data) return null;

    const d = entry.data;
    const cleanName = normalizeGameTitle(query.name || "");
    if (cleanName && (!titlesMatchStrict(cleanName, d.name) || !validateMetadataCandidate(cleanName, d.name, d.developers, d.publishers))) {
      return null;
    }

    return {
      name: d.name,
      shortDescription: d.short_description,
      detailedDescription: d.detailed_description,
      releaseDate: d.release_date?.date,
      developers: d.developers || [],
      publishers: d.publishers || [],
      genres: (d.genres || []).map((g: any) => g.description),
      categories: (d.categories || []).map((c: any) => c.description),
      criticScore: d.metacritic?.score,
      rating: d.metacritic?.score,
      websiteUrl: d.website || undefined,
      ageRating: d.required_age ? `${d.required_age}+` : undefined,
      headerImage: d.header_image,
      capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/library_600x900_2x.jpg`,
      heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/library_hero.jpg`,
      logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/logo.png`,
      screenshots: (d.screenshots || []).map((s: any) => s.path_full),
      movies: (d.movies || []).map((m: any) => m.mp4?.max || m.webm?.max).filter(Boolean),
    };
  } catch {
    return null;
  }
}

/**
 * Global helper querying GOG Catalog v1 API via Rust HTTP client.
 */
export async function queryGogCatalog(query: { name?: string; platform?: string }): Promise<PluginMetadataResult | null> {
  const cleanName = normalizeGameTitle(query.name || "");
  if (!cleanName) return null;

  try {
    const url = `https://catalog.gog.com/v1/catalog?limit=5&query=like:${encodeURIComponent(cleanName)}&order=desc:score&productType=in:game,pack`;
    const data = await api.fetchExternalJson<any>(url);
    const products = data?.products;
    if (!Array.isArray(products) || products.length === 0) return null;

    const p = products.find((prod: any) => {
      const title = prod.title || "";
      const devs = Array.isArray(prod.developers) ? prod.developers : [];
      const pubs = Array.isArray(prod.publishers) ? prod.publishers : [];
      return titlesMatchStrict(cleanName, title) && validateMetadataCandidate(cleanName, title, devs, pubs);
    });
    if (!p) return null;

    return {
      name: p.title || cleanName,
      releaseDate: p.releaseDate || undefined,
      developers: Array.isArray(p.developers) ? p.developers : [],
      publishers: Array.isArray(p.publishers) ? p.publishers : [],
      genres: Array.isArray(p.genres) ? p.genres.map((g: any) => g.name || g).filter(Boolean) : [],
      tags: Array.isArray(p.tags) ? p.tags.map((t: any) => t.name || t).filter(Boolean) : [],
      rating: typeof p.reviewsRating === "number" ? p.reviewsRating : undefined,
      capsuleImage: p.coverVertical || undefined,
      heroImage: p.coverHorizontal || undefined,
      headerImage: p.coverHorizontal || undefined,
      screenshots: Array.isArray(p.screenshots)
        ? p.screenshots.map((s: string) => s.replace("{formatter}", "ggvgl_2x"))
        : [],
    };
  } catch {
    return null;
  }
}

/**
 * Global helper querying PCGamingWiki Cargo API via Rust HTTP client.
 */
export async function queryPcgamingWiki(query: { name?: string }): Promise<PluginMetadataResult | null> {
  const cleanName = normalizeGameTitle(query.name || "");
  if (!cleanName) return null;

  try {
    const escaped = cleanName.replace(/"/g, "");
    const url = `https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game&fields=Infobox_game._pageName=Page,Infobox_game.Developers,Infobox_game.Publishers,Infobox_game.Released,Infobox_game.Genres,Infobox_game.Engines&where=Infobox_game._pageName%20LIKE%20%22${encodeURIComponent(escaped)}%22&format=json&limit=1`;
    const data = await api.fetchExternalJson<any>(url);
    const row = data?.cargoquery?.[0]?.title;
    if (!row) return null;

    const splitList = (v?: string) =>
      v
        ? v
            .split(",")
            .map((s) => s.trim())
            .filter(Boolean)
        : [];

    const developers = splitList(row.Developers);
    const publishers = splitList(row.Publishers);
    const pageName = row.Page || cleanName;

    if (!titlesMatchStrict(cleanName, pageName) || !validateMetadataCandidate(cleanName, pageName, developers, publishers)) {
      return null;
    }

    const releaseDate = row.Released ? String(row.Released).split(";")[0]?.trim() : undefined;
    return {
      name: pageName,
      developers,
      publishers,
      genres: splitList(row.Genres),
      releaseDate,
    };
  } catch {
    return null;
  }
}

/**
 * Global helper querying SteamGridDB v2 API when an API key is configured.
 */
export async function querySteamGridDb(
  query: { appId?: number | string; name?: string },
  config?: Record<string, any>
): Promise<PluginArtworkResult | null> {
  const apiKey = (config?.apiKey || "").trim();
  if (!apiKey) return null;
  const cleanName = normalizeGameTitle(query.name || "");
  if (!cleanName) return null;

  const headers = { Authorization: `Bearer ${apiKey}` };
  try {
    const search = await api.fetchExternalJson<any>(
      `https://www.steamgriddb.com/api/v2/search/autocomplete/${encodeURIComponent(cleanName)}`,
      headers
    );
    const matchingGame = search?.data?.find((d: any) =>
      titlesMatchStrict(cleanName, d.name) && validateMetadataCandidate(cleanName, d.name)
    );
    const gameId = matchingGame?.id;
    if (!gameId) return null;

    const [grids, heroes, logos] = await Promise.all([
      api
        .fetchExternalJson<any>(`https://www.steamgriddb.com/api/v2/grids/game/${gameId}?dimensions=600x900`, headers)
        .catch(() => null),
      api
        .fetchExternalJson<any>(`https://www.steamgriddb.com/api/v2/heroes/game/${gameId}`, headers)
        .catch(() => null),
      api
        .fetchExternalJson<any>(`https://www.steamgriddb.com/api/v2/logos/game/${gameId}`, headers)
        .catch(() => null),
    ]);

    return {
      capsuleImage: grids?.data?.[0]?.url || undefined,
      heroImage: heroes?.data?.[0]?.url || undefined,
      logoImage: logos?.data?.[0]?.url || undefined,
    };
  } catch {
    return null;
  }
}

/**
 * Global helper querying IGDB v4 API when Twitch Client ID & Secret are configured.
 */
export async function queryIgdb(
  query: { name?: string },
  config?: Record<string, any>
): Promise<PluginMetadataResult | null> {
  const clientId = (config?.clientId || "").trim();
  const clientSecret = (config?.clientSecret || "").trim();
  if (!clientId || !clientSecret) return null;
  const cleanName = normalizeGameTitle(query.name || "");
  if (!cleanName) return null;

  try {
    const res = await fetch(
      `https://id.twitch.tv/oauth2/token?client_id=${encodeURIComponent(clientId)}&client_secret=${encodeURIComponent(clientSecret)}&grant_type=client_credentials`,
      { method: "POST" }
    );
    if (!res.ok) return null;
    const tokenData = await res.json();
    const accessToken = tokenData?.access_token;
    if (!accessToken) return null;

    const igdbRes = await fetch("https://api.igdb.com/v4/games", {
      method: "POST",
      headers: {
        "Client-ID": clientId,
        Authorization: `Bearer ${accessToken}`,
        "Content-Type": "text/plain",
      },
      body: `search "${cleanName.replace(/"/g, "")}"; fields name,summary,first_release_date,total_rating,genres.name,cover.image_id,artworks.image_id,screenshots.image_id,involved_companies.company.name,involved_companies.developer,involved_companies.publisher; limit 5;`,
    });
    if (!igdbRes.ok) return null;
    const list = await igdbRes.json();
    if (!Array.isArray(list) || list.length === 0) return null;

    let g: any = null;
    let developers: string[] = [];
    let publishers: string[] = [];

    for (const item of list) {
      const devs: string[] = [];
      const pubs: string[] = [];
      for (const ic of item.involved_companies || []) {
        const cname = ic?.company?.name;
        if (!cname) continue;
        if (ic.developer) devs.push(cname);
        if (ic.publisher) pubs.push(cname);
      }
      if (titlesMatchStrict(cleanName, item.name || "") && validateMetadataCandidate(cleanName, item.name || "", devs, pubs)) {
        g = item;
        developers = devs;
        publishers = pubs;
        break;
      }
    }
    if (!g) return null;

    const releaseYear = g.first_release_date
      ? new Date(g.first_release_date * 1000).getUTCFullYear()
      : undefined;
    const releaseDate = g.first_release_date
      ? new Date(g.first_release_date * 1000).toISOString().slice(0, 10)
      : undefined;

    return {
      name: g.name || cleanName,
      shortDescription: g.summary || undefined,
      releaseDate,
      releaseYear,
      developers,
      publishers,
      genres: (g.genres || []).map((x: any) => x.name).filter(Boolean),
      rating: typeof g.total_rating === "number" ? Math.round(g.total_rating) : undefined,
      capsuleImage: g.cover?.image_id
        ? `https://images.igdb.com/igdb/image/upload/t_cover_big/${g.cover.image_id}.jpg`
        : undefined,
      heroImage: g.artworks?.[0]?.image_id
        ? `https://images.igdb.com/igdb/image/upload/t_1080p/${g.artworks[0].image_id}.jpg`
        : g.screenshots?.[0]?.image_id
          ? `https://images.igdb.com/igdb/image/upload/t_1080p/${g.screenshots[0].image_id}.jpg`
          : undefined,
      screenshots: (g.screenshots || [])
        .map((s: any) =>
          s.image_id ? `https://images.igdb.com/igdb/image/upload/t_screenshot_big/${s.image_id}.jpg` : null
        )
        .filter(Boolean),
    };
  } catch {
    return null;
  }
}

/**
 * Global helper testing connection credentials for any metadata provider (`metadata-universal`, `metadata-screenscraper`, `metadata-igdb`, `metadata-steamgriddb`).
 */
export async function testMetadataProviderConnection(
  providerId: string,
  config: Record<string, any>
): Promise<TestConnectionResult> {
  if (providerId === "metadata-steamgriddb") {
    const key = (config?.apiKey || "").trim();
    if (!key) {
      return {
        success: false,
        message: "Please enter a SteamGridDB v2 API key to verify authentication.",
      };
    }
    try {
      const data = await api.fetchExternalJson<any>(
        "https://www.steamgriddb.com/api/v2/search/autocomplete/Half-Life",
        { Authorization: `Bearer ${key}` }
      );
      if (data?.success) {
        return { success: true, message: "SteamGridDB API key verified and ready!" };
      }
      return { success: false, message: "SteamGridDB rejected the API key." };
    } catch (e: any) {
      return { success: false, message: `SteamGridDB check failed: ${e?.message || e}` };
    }
  }

  if (providerId === "metadata-igdb") {
    const clientId = (config?.clientId || "").trim();
    const clientSecret = (config?.clientSecret || "").trim();
    if (!clientId || !clientSecret) {
      return {
        success: false,
        message: "Both Twitch Client ID and Client Secret are required for IGDB OAuth2.",
      };
    }
    try {
      const res = await fetch(
        `https://id.twitch.tv/oauth2/token?client_id=${encodeURIComponent(clientId)}&client_secret=${encodeURIComponent(clientSecret)}&grant_type=client_credentials`,
        { method: "POST" }
      );
      if (res.ok) {
        return { success: true, message: "IGDB Twitch OAuth2 credentials verified!" };
      }
      return { success: false, message: `Twitch OAuth2 returned HTTP ${res.status}.` };
    } catch (e: any) {
      return { success: false, message: `IGDB OAuth2 check failed: ${e?.message || e}` };
    }
  }

  if (providerId === "metadata-screenscraper") {
    const userId = (config?.userId || "").trim();
    const userPassword = (config?.userPassword || "").trim();
    if (!userId || !userPassword) {
      return {
        success: true,
        message: "Keyless Libretro Thumbnails CDN & Wikipedia/Wikidata console metadata active (ScreenScraper account optional).",
      };
    }
    return {
      success: true,
      message: `ScreenScraper member credentials configured for account '${userId}'.`,
    };
  }

  return {
    success: true,
    message:
      "Universal Metadata Engine active (Local Inspector + Libretro Thumbnails CDN + Steam Store + GOG Catalog + Wikipedia/Wikidata + PCGamingWiki).",
  };
}

/**
 * Global helper resolving comprehensive game metadata across the Universal Metadata Engine waterfall.
 */
export async function resolveUniversalMetadata(
  query: { appId?: number | string; name?: string; platform?: string },
  config?: Record<string, any>
): Promise<PluginMetadataResult | null> {
  // 0. Primary authoritative check: HELIX Gaming DB Three-Tier REST API
  const helixMeta = await queryHelixGamingDb(query, {
    customBase: config?.endpoint_url,
    includeAdult: config?.include_adult_content !== false,
  });
  if (helixMeta && (helixMeta.shortDescription || helixMeta.capsuleImage)) {
    return helixMeta;
  }

  // 1. Dynamic Direct Search: Steam Storefront (for PC titles)
  const steamMeta = await querySteamStoreFront(query, config);
  if (steamMeta && (steamMeta.shortDescription || steamMeta.capsuleImage)) {
    return steamMeta;
  }

  // 2. Dynamic Direct Search: GOG Catalog v1 API
  const gogMeta = await queryGogCatalog(query);

  // 3. Dynamic Direct Search: PCGamingWiki Cargo API
  const pcgwMeta = await queryPcgamingWiki(query);

  // 4. Dynamic Direct Search: IGDB if configured
  const igdbMeta = await queryIgdb(query, config);

  if (!gogMeta && !pcgwMeta && !igdbMeta) {
    const backup = backupMetadataForTitle(query.name || "");
    if (backup) {
      return backup;
    }
    return null;
  }

  const result: PluginMetadataResult = {
    name: igdbMeta?.name || gogMeta?.name || pcgwMeta?.name || query.name,
    shortDescription: igdbMeta?.shortDescription || gogMeta?.shortDescription || undefined,
    detailedDescription: igdbMeta?.detailedDescription || undefined,
    releaseDate: igdbMeta?.releaseDate || gogMeta?.releaseDate || pcgwMeta?.releaseDate || undefined,
    releaseYear: igdbMeta?.releaseYear || gogMeta?.releaseYear || undefined,
    developers:
      (igdbMeta?.developers?.length ? igdbMeta.developers : null) ||
      (pcgwMeta?.developers?.length ? pcgwMeta.developers : null) ||
      gogMeta?.developers ||
      [],
    publishers:
      (igdbMeta?.publishers?.length ? igdbMeta.publishers : null) ||
      (pcgwMeta?.publishers?.length ? pcgwMeta.publishers : null) ||
      gogMeta?.publishers ||
      [],
    genres:
      (igdbMeta?.genres?.length ? igdbMeta.genres : null) ||
      (gogMeta?.genres?.length ? gogMeta.genres : null) ||
      pcgwMeta?.genres ||
      [],
    categories: igdbMeta?.categories || [],
    tags: gogMeta?.tags || igdbMeta?.tags || [],
    rating: igdbMeta?.rating ?? gogMeta?.rating,
    ageRating: igdbMeta?.ageRating,
    capsuleImage: igdbMeta?.capsuleImage || gogMeta?.capsuleImage,
    heroImage: igdbMeta?.heroImage || gogMeta?.heroImage,
    logoImage: igdbMeta?.logoImage,
    headerImage: gogMeta?.headerImage || igdbMeta?.headerImage,
    screenshots: igdbMeta?.screenshots?.length ? igdbMeta.screenshots : gogMeta?.screenshots || [],
    movies: igdbMeta?.movies || [],
    soundtracks: igdbMeta?.soundtracks || [],
  };

  // Supplement any missing fields with internal hand-rolled / known developer backup
  const backup = backupMetadataForTitle(query.name || "");
  if (backup) {
    if (!result.developers?.length && backup.developers?.length) result.developers = [...backup.developers];
    if (!result.publishers?.length && backup.publishers?.length) result.publishers = [...backup.publishers];
    if (!result.genres?.length && backup.genres?.length) result.genres = [...backup.genres];
    if (!result.categories?.length && backup.categories?.length) result.categories = [...backup.categories];
    if (!result.tags?.length && backup.tags?.length) result.tags = [...backup.tags];
    if (!result.capsuleImage && backup.capsuleImage) result.capsuleImage = backup.capsuleImage;
    if (!result.heroImage && backup.heroImage) result.heroImage = backup.heroImage;
    if (!result.logoImage && backup.logoImage) result.logoImage = backup.logoImage;
    if (!result.headerImage && backup.headerImage) result.headerImage = backup.headerImage;
    if (!result.iconImage && backup.iconImage) result.iconImage = backup.iconImage;
    if (!result.shortDescription && backup.shortDescription) result.shortDescription = backup.shortDescription;
    if (!result.detailedDescription && backup.detailedDescription) result.detailedDescription = backup.detailedDescription;
    if (!result.releaseDate && backup.releaseDate) result.releaseDate = backup.releaseDate;
    if (!result.releaseYear && backup.releaseYear) result.releaseYear = backup.releaseYear;
    if (!result.ageRating && backup.ageRating) result.ageRating = backup.ageRating;
    if ((!result.screenshots || result.screenshots.length === 0) && backup.screenshots?.length) {
      result.screenshots = [...backup.screenshots];
    }
    if ((!result.movies || result.movies.length === 0) && backup.movies?.length) {
      result.movies = [...backup.movies];
    }
    if ((!result.soundtracks || result.soundtracks.length === 0) && backup.soundtracks?.length) {
      result.soundtracks = [...backup.soundtracks];
    }
  }

  return result;
}

/**
 * Global helper resolving high-resolution artwork (2:3 capsule, 16:9 hero banner, transparent logo) across providers.
 */
export async function resolveUniversalArtwork(
  query: { appId?: number | string; name?: string; platform?: string },
  config?: Record<string, any>
): Promise<PluginArtworkResult | null> {
  // 1. Primary authoritative check: HELIX Gaming DB Three-Tier REST API
  const helixArt = await queryHelixGamingDbArtwork(query, {
    customBase: config?.endpoint_url,
    includeAdult: config?.include_adult_content !== false,
  });
  if (helixArt && (helixArt.capsuleImage || helixArt.heroImage || helixArt.logoImage)) {
    return helixArt;
  }

  // 2. Check SteamGridDB if user configured an API key
  const sgdb = await querySteamGridDb(query, config);
  if (sgdb && (sgdb.capsuleImage || sgdb.heroImage || sgdb.logoImage)) {
    return sgdb;
  }

  // 3. Dynamic Direct Search: Check Steam CDN for PC titles
  const plat = (query.platform || "").toLowerCase();
  const isConsoleRom =
    plat &&
    ![
      "steam",
      "gog",
      "epic",
      "itch",
      "xbox",
      "ea",
      "ubisoft",
      "battlenet",
      "amazon",
      "rockstar",
      "standalone",
      "other",
      "pc",
    ].includes(plat);

  if (!isConsoleRom) {
    const resolvedId = await resolveSteamAppId(query.appId, query.name);
    if (resolvedId) {
      return {
        capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/library_600x900_2x.jpg`,
        heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/library_hero.jpg`,
        logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/logo.png`,
        iconImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${resolvedId}/header.jpg`,
      };
    }
  }

  return null;
}

/**
 * Universal Metadata Plugin (`metadata-universal`) consolidating Local Manifest & PE Inspector,
 * Libretro Thumbnails CDN, Steam Store, GOG Catalog, Wikipedia/Wikidata, PCGamingWiki,
 * SteamGridDB, IGDB, and ScreenScraper into a single automatic enrichment engine.
 */
export const metadataUniversalPlugin: UniversalPlugin = {
  id: "metadata-universal",
  name: "Universal Metadata & Artwork Engine",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description:
    "Automatically enriches all PC, indie, standalone, and EmulationStation ROM titles with high-resolution 2:3 capsules, 16:9 hero banners, transparent logos, synopses, developers, publishers, genres, and release dates.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: [
    "auto_enrichment",
    "local_manifest_inspector",
    "libretro_thumbnails_cdn",
    "steam_store",
    "gog_catalog",
    "wikidata_mediawiki",
    "pcgamingwiki",
    "steamgriddb",
    "igdb",
  ],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "autoEnhanceOnScan",
          label: "Automatically Enhance Metadata & Artwork on Scan",
          type: "boolean",
          default: true,
          description: "Automatically resolve missing descriptions, developers, genres, and artwork in the background whenever games are imported or selected."
        },
        {
          key: "countryCode",
          label: "Storefront Region / Country Code",
          type: "text",
          default: "us",
          description: "Two-letter ISO country code for localized store descriptions.",
          placeholder: "us"
        },
        {
          key: "language",
          label: "Preferred Metadata Language",
          type: "text",
          default: "english",
          description: "Preferred language for game synopses and genre tags.",
          placeholder: "english"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    return testMetadataProviderConnection("metadata-universal", config);
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    return resolveUniversalMetadata(query, config);
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    return resolveUniversalArtwork(query, config);
  },
};
