import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const gogCatalogPlugin: UniversalPlugin = {
  id: "metadata-gog",
  name: "GOG Catalog Metadata & CDN Artwork",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Official GOG catalog endpoints for store artwork, game summaries, DRM-free releases, and system requirements.",
  homepage: "https://www.gog.com",
  features: ["store_artwork", "drm_free_meta", "system_requirements"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: []
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    try {
      const res = await fetch("https://embed.gog.com/games/ajax/filtered?mediaType=game&search=witcher");
      if (res.ok) {
        return { success: true, message: "GOG public catalog endpoints connected successfully!" };
      }
      return { success: false, message: `GOG returned HTTP error ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Failed to connect to GOG: ${err?.message || err}` };
    }
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch(`https://embed.gog.com/games/ajax/filtered?mediaType=game&search=${encodeURIComponent(term)}`);
      if (!res.ok) return null;
      const data = await res.json();
      const product = data?.products?.[0];
      if (!product) return null;

      const rawImg = product.image ? (product.image.startsWith("//") ? `https:${product.image}` : product.image) : null;
      const capsule = rawImg ? `${rawImg}_product_card_v2_mobile_slider_639.jpg` : undefined;
      const hero = rawImg ? `${rawImg}_bg_crop_1920x1080.jpg` : undefined;
      const header = rawImg ? `${rawImg}.jpg` : undefined;

      const score = typeof product.rating === "number" ? Math.round((product.rating / 50) * 100) : undefined;
      const relDate = product.releaseDate ? new Date(product.releaseDate * 1000).toISOString().split("T")[0] : undefined;
      const relYear = product.releaseDate ? new Date(product.releaseDate * 1000).getFullYear() : undefined;

      return {
        name: product.title || term,
        releaseDate: relDate,
        releaseYear: relYear,
        developers: product.developers?.length ? product.developers : undefined,
        publishers: product.publishers?.length ? product.publishers : undefined,
        genres: product.genres || [],
        rating: score,
        criticScore: score,
        capsuleImage: capsule,
        heroImage: hero,
        headerImage: header,
        websiteUrl: product.url ? `https://www.gog.com${product.url}` : undefined
      };
    } catch {
      return null;
    }
  },

  async queryArtwork(query): Promise<PluginArtworkResult | null> {
    const meta = await this.queryMetadata!(query, {});
    if (!meta) return null;
    return {
      capsuleImage: meta.capsuleImage,
      heroImage: meta.heroImage,
      headerImage: meta.headerImage
    };
  }
};
