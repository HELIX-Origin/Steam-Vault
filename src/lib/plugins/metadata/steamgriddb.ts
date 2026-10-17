import type { UniversalPlugin, TestConnectionResult, PluginArtworkResult, PluginMetadataResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const steamgriddbPlugin: UniversalPlugin = {
  id: "metadata-steamgriddb",
  name: "SteamGridDB Community Artwork Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Provides access to community-crafted custom capsule covers, hero banners, and logos via the SteamGridDB API.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher-Plugins",
  features: ["custom_artwork", "community_grids", "heroes", "logos", "icons"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: true,
      fields: [
        {
          key: "apiKey",
          label: "SteamGridDB API Key",
          type: "password",
          required: true,
          description: "Personal API token used to fetch custom high-resolution covers, heroes, and logos from SteamGridDB.",
          placeholder: "Paste your SteamGridDB API key here…",
          helpUrl: "https://www.steamgriddb.com/profile/preferences/api"
        },
        {
          key: "dimensions",
          label: "Preferred Capsule Dimensions",
          type: "select",
          default: "600x900",
          description: "Preferred aspect ratio for game library cover cards.",
          options: [
            { label: "Vertical Posters (600x900)", value: "600x900" },
            { label: "Legacy Grids (460x215)", value: "460x215" }
          ]
        },
        {
          key: "allowNsfw",
          label: "Allow NSFW Submissions",
          type: "boolean",
          default: false,
          description: "Include adult or mature submissions in artwork search queries."
        },
        {
          key: "allowHumor",
          label: "Allow Humor & Parody Submissions",
          type: "boolean",
          default: true,
          description: "Include parody, meme, and joke artwork covers."
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const key = (config?.apiKey ?? "").trim();
    if (!key) {
      return { success: false, message: "SteamGridDB API key is required." };
    }
    try {
      const res = await fetch("https://www.steamgriddb.com/api/v2/search/autocomplete/portal", {
        headers: {
          Authorization: `Bearer ${key}`
        }
      });
      if (res.ok) {
        return { success: true, message: "Successfully connected to SteamGridDB API!" };
      }
      if (res.status === 401 || res.status === 403) {
        return { success: false, message: "Invalid SteamGridDB API key. Check credentials on steamgriddb.com." };
      }
      return { success: false, message: `SteamGridDB returned HTTP error ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Connection error: ${err?.message || err}` };
    }
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    const key = (config?.apiKey ?? "").trim();
    if (!key) return null;

    const term = query.name;
    if (!term) return null;

    try {
      const searchRes = await fetch(
        `https://www.steamgriddb.com/api/v2/search/autocomplete/${encodeURIComponent(term)}`,
        { headers: { Authorization: `Bearer ${key}` } }
      );
      if (!searchRes.ok) return null;
      const searchData = await searchRes.json();
      const firstGame = searchData?.data?.[0];
      if (!firstGame?.id) return null;

      const gameId = firstGame.id;
      const [gridsRes, heroesRes, logosRes, iconsRes] = await Promise.all([
        fetch(`https://www.steamgriddb.com/api/v2/grids/game/${gameId}?dimensions=600x900`, {
          headers: { Authorization: `Bearer ${key}` }
        }),
        fetch(`https://www.steamgriddb.com/api/v2/heroes/game/${gameId}`, {
          headers: { Authorization: `Bearer ${key}` }
        }),
        fetch(`https://www.steamgriddb.com/api/v2/logos/game/${gameId}`, {
          headers: { Authorization: `Bearer ${key}` }
        }),
        fetch(`https://www.steamgriddb.com/api/v2/icons/game/${gameId}`, {
          headers: { Authorization: `Bearer ${key}` }
        })
      ]);

      const gridsData = gridsRes.ok ? await gridsRes.json() : null;
      const heroesData = heroesRes.ok ? await heroesRes.json() : null;
      const logosData = logosRes.ok ? await logosRes.json() : null;
      const iconsData = iconsRes.ok ? await iconsRes.json() : null;

      return {
        capsuleImage: gridsData?.data?.[0]?.url ?? null,
        heroImage: heroesData?.data?.[0]?.url ?? null,
        logoImage: logosData?.data?.[0]?.url ?? null,
        iconImage: iconsData?.data?.[0]?.url ?? null
      };
    } catch {
      return null;
    }
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const key = (config?.apiKey ?? "").trim();
    if (!key) return null;

    const term = query.name;
    if (!term) return null;

    try {
      const searchRes = await fetch(
        `https://www.steamgriddb.com/api/v2/search/autocomplete/${encodeURIComponent(term)}`,
        { headers: { Authorization: `Bearer ${key}` } }
      );
      if (!searchRes.ok) return null;
      const searchData = await searchRes.json();
      const firstGame = searchData?.data?.[0];
      if (!firstGame?.id) return null;

      const artwork = await this.queryArtwork!(query, config);

      return {
        name: firstGame.name,
        releaseYear: firstGame.release_date ? new Date(firstGame.release_date * 1000).getFullYear() : undefined,
        capsuleImage: artwork?.capsuleImage ?? undefined,
        heroImage: artwork?.heroImage ?? undefined,
        logoImage: artwork?.logoImage ?? undefined,
        iconImage: artwork?.iconImage ?? undefined,
        websiteUrl: `https://www.steamgriddb.com/game/${firstGame.id}`
      };
    } catch {
      return null;
    }
  }
};
