import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const igdbPlugin: UniversalPlugin = {
  id: "metadata-igdb",
  name: "IGDB Comprehensive Gaming Metadata Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Queries the Internet Game Database (IGDB) by Twitch for global taxonomy, developer lists, themes, and critic ratings.",
  homepage: "https://www.igdb.com",
  features: ["game_taxonomy", "critic_reviews", "developer_credits", "franchises"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: true,
      fields: [
        {
          key: "clientId",
          label: "Twitch / IGDB Client ID",
          type: "text",
          required: true,
          description: "Registered Twitch Developer Application Client ID.",
          placeholder: "Paste Twitch Client ID here…",
          helpUrl: "https://dev.twitch.tv/console/apps"
        },
        {
          key: "clientSecret",
          label: "Twitch / IGDB Client Secret",
          type: "password",
          required: true,
          description: "Twitch Developer Application Client Secret generated from the developer console.",
          placeholder: "Paste Twitch Client Secret here…",
          helpUrl: "https://dev.twitch.tv/console/apps"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const clientId = (config?.clientId ?? "").trim();
    const clientSecret = (config?.clientSecret ?? "").trim();

    if (!clientId || !clientSecret) {
      return { success: false, message: "Both Twitch Client ID and Client Secret are required." };
    }

    try {
      const authUrl = `https://id.twitch.tv/oauth2/token?client_id=${clientId}&client_secret=${clientSecret}&grant_type=client_credentials`;
      const res = await fetch(authUrl, { method: "POST" });
      if (res.ok) {
        const data = await res.json();
        if (data?.access_token) {
          return { success: true, message: "Authenticated with Twitch / IGDB OAuth API successfully!" };
        }
      }
      return { success: false, message: `Twitch authentication failed (HTTP ${res.status}). Verify your Client ID and Secret.` };
    } catch (err: any) {
      return { success: false, message: `Network error connecting to Twitch OAuth: ${err?.message || err}` };
    }
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const clientId = (config?.clientId ?? "").trim();
    const clientSecret = (config?.clientSecret ?? "").trim();
    if (!clientId || !clientSecret || !query.name) return null;

    try {
      const authUrl = `https://id.twitch.tv/oauth2/token?client_id=${clientId}&client_secret=${clientSecret}&grant_type=client_credentials`;
      const authRes = await fetch(authUrl, { method: "POST" });
      if (!authRes.ok) return null;
      const authData = await authRes.json();
      const token = authData?.access_token;
      if (!token) return null;

      const body = `search "${query.name.replace(/"/g, '\\"')}"; fields name, summary, storyline, first_release_date, genres.name, themes.name, game_modes.name, involved_companies.developer, involved_companies.publisher, involved_companies.company.name, cover.url, artworks.url, screenshots.url, videos.video_id, total_rating, rating, websites.url, collection.name, franchises.name; limit 1;`;
      const igdbRes = await fetch("https://api.igdb.com/v4/games", {
        method: "POST",
        headers: {
          "Client-ID": clientId,
          Authorization: `Bearer ${token}`
        },
        body
      });
      if (!igdbRes.ok) return null;
      const games = await igdbRes.json();
      const g = games?.[0];
      if (!g) return null;

      const coverUrl = g.cover?.url ? `https:${g.cover.url.replace("t_thumb", "t_cover_big")}` : undefined;
      const heroUrl = g.artworks?.[0]?.url ? `https:${g.artworks[0].url.replace("t_thumb", "t_1080p")}` : undefined;
      const screenshots = (g.screenshots || []).map((s: any) => `https:${s.url.replace("t_thumb", "t_1080p")}`);
      const movies = (g.videos || []).map((v: any) => `https://www.youtube.com/watch?v=${v.video_id}`);

      const developers = (g.involved_companies || [])
        .filter((c: any) => c.developer)
        .map((c: any) => c.company?.name)
        .filter(Boolean);

      const publishers = (g.involved_companies || [])
        .filter((c: any) => c.publisher)
        .map((c: any) => c.company?.name)
        .filter(Boolean);

      return {
        name: g.name,
        shortDescription: g.summary,
        detailedDescription: g.storyline || g.summary,
        releaseDate: g.first_release_date ? new Date(g.first_release_date * 1000).toISOString().split("T")[0] : undefined,
        releaseYear: g.first_release_date ? new Date(g.first_release_date * 1000).getFullYear() : undefined,
        developers: developers.length > 0 ? developers : undefined,
        publishers: publishers.length > 0 ? publishers : undefined,
        genres: (g.genres || []).map((x: any) => x.name),
        tags: (g.themes || []).map((x: any) => x.name),
        categories: (g.game_modes || []).map((x: any) => x.name),
        rating: g.total_rating ?? g.rating,
        criticScore: g.total_rating ? Math.round(g.total_rating) : undefined,
        userScore: g.rating ? Math.round(g.rating) : undefined,
        series: g.collection?.name ?? g.franchises?.[0]?.name,
        capsuleImage: coverUrl,
        headerImage: coverUrl,
        heroImage: heroUrl,
        screenshots: screenshots.length > 0 ? screenshots : undefined,
        movies: movies.length > 0 ? movies : undefined,
        websiteUrl: g.websites?.[0]?.url
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
      headerImage: meta.headerImage,
      screenshots: meta.screenshots
    };
  }
};
