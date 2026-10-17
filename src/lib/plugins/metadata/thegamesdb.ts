import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const thegamesdbPlugin: UniversalPlugin = {
  id: "metadata-thegamesdb",
  name: "TheGamesDB Retro Metadata Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Comprehensive community video game database specializing in retro console ROMs, box art, fanart heroes, clear logos, and release taxonomy.",
  homepage: "https://thegamesdb.net",
  features: ["retro_metadata", "boxart", "fanart", "clearlogos", "taxonomy", "screenshots"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "apiKey",
          label: "TheGamesDB API Key",
          type: "password",
          required: false,
          default: "",
          description: "Optional personal API key for high-throughput retro game scraping from thegamesdb.net (requires logging in to your thegamesdb.net account to view your key).",
          placeholder: "Paste TheGamesDB API key here…",
          helpUrl: "https://api.thegamesdb.net/key.php"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const apiKey = (config?.apiKey || "").trim();
    if (!apiKey) {
      return { success: true, message: "Using keyless public access. Optional API key can be supplied for higher rate limits." };
    }
    try {
      const url = `https://api.thegamesdb.net/v1/Games/ByGameName?name=Super+Mario+World&apikey=${encodeURIComponent(apiKey)}`;
      const res = await fetch(url, { signal: AbortSignal.timeout(6000) });
      if (res.ok) {
        return { success: true, message: "Authenticated with TheGamesDB API successfully!" };
      }
      return { success: false, message: `TheGamesDB request returned HTTP ${res.status}. Check your API key.` };
    } catch (err: any) {
      return { success: false, message: `Connection test error: ${err?.message || err}` };
    }
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const name = query.name?.trim();
    if (!name) return null;

    const apiKey = (config?.apiKey || "").trim();
    const cleanName = name
      .replace(/\(USA\)|\(Europe\)|\(Japan\)|\(World\)|\(Rev \d+\)|\[!\]/gi, "")
      .replace(/\.(nes|snes|z64|n64|iso|rvz|gcz|ciso|wbfs|bin|cue|chd|gba|gbc|gb|nds|3ds|nsp|xci)$/i, "")
      .trim();

    try {
      let url = `https://api.thegamesdb.net/v1/Games/ByGameName?name=${encodeURIComponent(cleanName)}`;
      if (apiKey) {
        url += `&apikey=${encodeURIComponent(apiKey)}`;
      }

      const res = await fetch(url, { signal: AbortSignal.timeout(8000) });
      if (!res.ok) return null;
      const data = await res.json();
      const game = data?.data?.games?.[0];
      if (!game) return null;

      const baseCdn = data?.pages?.image_cdn || "https://cdn.thegamesdb.net/images/original/";

      // Parse box art, fanart, and logo images if available in images payload
      let boxArt: string | undefined;
      let fanart: string | undefined;
      let logo: string | undefined;
      const screenshots: string[] = [];

      const images = data?.include?.boxart?.data?.[game.id] || [];
      if (Array.isArray(images)) {
        for (const img of images) {
          const fullUrl = img.filename ? `${baseCdn}${img.filename}` : undefined;
          if (!fullUrl) continue;
          if (img.side === "front" && !boxArt) {
            boxArt = fullUrl;
          } else if (img.type === "fanart" && !fanart) {
            fanart = fullUrl;
          } else if (img.type === "clearlogo" && !logo) {
            logo = fullUrl;
          } else if (img.type === "screenshot" && screenshots.length < 10) {
            screenshots.push(fullUrl);
          }
        }
      }

      const releaseYear = game.release_date ? parseInt(game.release_date.split("-")[0], 10) : undefined;

      return {
        name: game.game_title || cleanName,
        shortDescription: game.overview?.slice(0, 300) || undefined,
        detailedDescription: game.overview || undefined,
        releaseDate: game.release_date || undefined,
        releaseYear: isNaN(releaseYear as number) ? undefined : releaseYear,
        developers: game.developers ? [game.developers].flat() : [],
        publishers: game.publishers ? [game.publishers].flat() : [],
        genres: game.genres ? [game.genres].flat() : [],
        players: game.players ? `${game.players} Player${game.players > 1 ? "s" : ""}` : undefined,
        rating: game.rating ? parseFloat(game.rating) * 10 : undefined,
        communityRating: game.rating ? parseFloat(game.rating) * 10 : undefined,
        capsuleImage: boxArt,
        heroImage: fanart,
        logoImage: logo,
        screenshots: screenshots.length > 0 ? screenshots : undefined
      };
    } catch {
      return null;
    }
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    const meta = await this.queryMetadata!(query, config);
    if (!meta) return null;
    return {
      capsuleImage: meta.capsuleImage,
      heroImage: meta.heroImage,
      logoImage: meta.logoImage
    };
  }
};
