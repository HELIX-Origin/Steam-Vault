import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const protondbPlugin: UniversalPlugin = {
  id: "metadata-protondb",
  name: "ProtonDB Linux & Steam Deck Tiers",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Community compatibility ratings (Native, Platinum, Gold, Silver, Bronze, Borked) and recommended launch commands.",
  homepage: "https://www.protondb.com",
  features: ["deck_verified", "proton_tier", "launch_tweaks"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "showTierBadge",
          label: "Show Proton / Deck Badges",
          type: "boolean",
          default: true,
          description: "Display Linux & Steam Deck compatibility medals on library game cards."
        }
      ]
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    try {
      const res = await fetch("https://www.protondb.com/api/v1/reports/summaries/570.json");
      if (res.ok) {
        return { success: true, message: "ProtonDB public summary API active and operational." };
      }
      return { success: false, message: `ProtonDB returned HTTP ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Failed to connect to ProtonDB: ${err?.message || err}` };
    }
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    const rawId = query.appId ?? query.platformId;
    if (!term && !rawId) return null;

    try {
      let appId = rawId && /^\d+$/.test(String(rawId)) ? String(rawId) : null;
      if (!appId && term) {
        const searchRes = await fetch(`https://store.steampowered.com/api/storesearch/?term=${encodeURIComponent(term)}&l=english&cc=US`);
        if (searchRes.ok) {
          const searchData = await searchRes.json();
          appId = searchData?.items?.[0]?.id?.toString() ?? null;
        }
      }

      if (!appId) return null;

      const res = await fetch(`https://www.protondb.com/api/v1/reports/summaries/${appId}.json`);
      if (!res.ok) return null;
      const data = await res.json();
      const tier: string = data?.tier || "unknown";
      const formattedTier = tier.charAt(0).toUpperCase() + tier.slice(1);
      const score = typeof data?.score === "number" ? Math.round(data.score * 100) : undefined;

      return {
        name: term || `Steam App ${appId}`,
        compatibilityTier: formattedTier,
        communityRating: score,
        tags: [`ProtonDB: ${formattedTier}`],
        websiteUrl: `https://www.protondb.com/app/${appId}`,
        extra: {
          protonTier: formattedTier,
          confidence: data?.confidence,
          reportCount: data?.total
        }
      };
    } catch {
      return null;
    }
  }
};
