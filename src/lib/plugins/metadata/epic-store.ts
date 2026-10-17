import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const epicStorePlugin: UniversalPlugin = {
  id: "metadata-epic",
  name: "Epic Games Store Catalog & Artwork",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Fetches Epic Games Store catalog manifests, promotional artwork covers, and release dates.",
  homepage: "https://store.epicgames.com",
  features: ["promotional_artwork", "catalog_manifests"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: []
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    try {
      const res = await fetch("https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US");
      if (res.ok) {
        return { success: true, message: "Epic Games Store catalog CDN endpoints connected successfully!" };
      }
      return { success: false, message: `Epic Games Store returned HTTP error ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Failed to connect to Epic Games Store: ${err?.message || err}` };
    }
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const res = await fetch("https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=en-US&country=US&allowCountries=US");
      if (!res.ok) return null;
      const data = await res.json();
      const elements: any[] = data?.data?.Catalog?.searchStore?.elements || [];
      const game = elements.find((e: any) => e.title?.toLowerCase().includes(term.toLowerCase()));
      if (!game) return null;

      const tallImg = game.keyImages?.find((img: any) => img.type === "OfferImageTall" || img.type === "Thumbnail")?.url;
      const wideImg = game.keyImages?.find((img: any) => img.type === "OfferImageWide" || img.type === "DieselStoreFrontWide")?.url;

      const devAttr = game.customAttributes?.find((a: any) => a.key === "developerName")?.value;
      const pubAttr = game.seller?.name || game.customAttributes?.find((a: any) => a.key === "publisherName")?.value;

      const relDate = game.effectiveDate?.split("T")[0];
      const relYear = game.effectiveDate ? new Date(game.effectiveDate).getFullYear() : undefined;

      const slug = game.productSlug || game.urlSlug;

      return {
        name: game.title || term,
        shortDescription: game.description,
        detailedDescription: game.description,
        releaseDate: relDate,
        releaseYear: relYear,
        developers: devAttr ? [devAttr] : undefined,
        publishers: pubAttr ? [pubAttr] : undefined,
        categories: (game.categories || []).map((c: any) => c.path).filter(Boolean),
        capsuleImage: tallImg,
        heroImage: wideImg,
        headerImage: wideImg,
        websiteUrl: slug ? `https://store.epicgames.com/p/${slug}` : undefined
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
