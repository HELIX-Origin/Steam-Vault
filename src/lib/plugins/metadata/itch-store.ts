import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const itchStorePlugin: UniversalPlugin = {
  id: "metadata-itch",
  name: "itch.io Indie Games Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Discovers indie game titles, developer profiles, screenshots, and account library synchronization from itch.io.",
  homepage: "https://itch.io",
  features: ["indie_meta", "screenshots", "developer_credits", "library_sync"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "apiKey",
          label: "itch.io API Key",
          type: "password",
          required: false,
          description: "Optional personal API key for accessing owned itch.io collections and private purchases.",
          placeholder: "Optional itch.io API key…",
          helpUrl: "https://itch.io/user/settings/api-keys"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const key = (config?.apiKey ?? "").trim();
    if (!key) {
      return { success: true, message: "Keyless mode active: Public itch.io storefront catalogs can be queried without an API key." };
    }
    try {
      const res = await fetch("https://itch.io/api/1/key/me", {
        headers: { Authorization: `Bearer ${key}` }
      });
      if (res.ok) {
        return { success: true, message: "itch.io API key authenticated successfully!" };
      }
      return { success: false, message: `itch.io API returned HTTP ${res.status}. Check key permissions.` };
    } catch (err: any) {
      return { success: false, message: `Connection error: ${err?.message || err}` };
    }
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch(`https://itch.io/search?q=${encodeURIComponent(term)}&format=json`);
      if (!res.ok) return null;
      const data = await res.json();
      const games: any[] = data?.games || [];
      const g = games[0];
      if (!g) return null;

      const dev = g.user?.name || g.user?.username;

      return {
        name: g.title || term,
        shortDescription: g.short_text,
        detailedDescription: g.short_text,
        developers: dev ? [dev] : undefined,
        publishers: dev ? [dev] : undefined,
        capsuleImage: g.cover_url,
        heroImage: g.cover_url,
        headerImage: g.cover_url,
        websiteUrl: g.url
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
      headerImage: meta.headerImage
    };
  }
};
