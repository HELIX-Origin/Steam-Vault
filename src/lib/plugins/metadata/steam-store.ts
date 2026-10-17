import type { UniversalPlugin, TestConnectionResult, PluginArtworkResult, PluginMetadataResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const steamStorePlugin: UniversalPlugin = {
  id: "metadata-steam",
  name: "Steam Store & Web API Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Official Steam Storefront metadata, CDN artwork, reviews, achievements, and Steam Web API synchronization.",
  homepage: "https://store.steampowered.com",
  features: ["store_metadata", "cdn_artwork", "achievements", "owned_library", "web_api"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "apiKey",
          label: "Steam Web API Key",
          type: "password",
          required: false,
          description: "Optional developer Web API key used for remote library synchronization and profile vanity URL resolution.",
          placeholder: "Optional personal Web API key…",
          helpUrl: "https://steamcommunity.com/dev/apikey"
        },
        {
          key: "countryCode",
          label: "Store Region / Country Code",
          type: "text",
          default: "us",
          description: "Two-letter ISO country code for localized prices, descriptions, and regional age ratings.",
          placeholder: "us"
        },
        {
          key: "language",
          label: "Store Description Language",
          type: "text",
          default: "english",
          description: "Preferred language for store synopsis and genre categorization.",
          placeholder: "english"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const key = (config?.apiKey ?? "").trim();
    if (!key) {
      return { success: true, message: "Keyless mode active: Public Steam storefront metadata & CDN artwork are available without an API key." };
    }
    try {
      const res = await fetch(`https://api.steampowered.com/ISteamWebAPIUtil/GetSupportedAPIList/v1/?key=${key}`);
      if (res.ok) {
        return { success: true, message: "Steam Web API Key authenticated successfully!" };
      }
      if (res.status === 403 || res.status === 401) {
        return { success: false, message: "Invalid Steam Web API Key. Verify your key at steamcommunity.com/dev/apikey." };
      }
      return { success: false, message: `Steam Web API returned HTTP ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Network error verifying key: ${err?.message || err}` };
    }
  },

  async queryArtwork(query): Promise<PluginArtworkResult | null> {
    const appId = query.appId;
    if (!appId) return null;
    return {
      capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/capsule_616x353.jpg`,
      heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`,
      logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`,
      iconImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`
    };
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const appId = query.appId;
    if (!appId) return null;
    const cc = (config?.countryCode || "us").trim();
    const lang = (config?.language || "english").trim();

    try {
      const res = await fetch(`https://store.steampowered.com/api/appdetails?appids=${appId}&cc=${cc}&l=${lang}`);
      if (!res.ok) return null;
      const data = await res.json();
      const entry = data[appId];
      if (!entry?.success || !entry?.data) return null;

      const d = entry.data;
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
        capsuleImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/capsule_616x353.jpg`,
        heroImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`,
        logoImage: `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`,
        screenshots: (d.screenshots || []).map((s: any) => s.path_full),
        movies: (d.movies || []).map((m: any) => m.mp4?.max || m.webm?.max).filter(Boolean)
      };
    } catch {
      return null;
    }
  }
};
