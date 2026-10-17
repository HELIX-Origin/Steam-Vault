import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const howlongtobeatPlugin: UniversalPlugin = {
  id: "metadata-howlongtobeat",
  name: "HowLongToBeat Completion Times",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Crowd-sourced gameplay completion metrics (Main Story, Main + Extra, Completionist) from HowLongToBeat.",
  homepage: "https://howlongtobeat.com",
  features: ["gameplay_length", "completionist_stats", "speedrun_times"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "preferredMetric",
          label: "Default Metric Display",
          type: "select",
          default: "main",
          description: "Primary completion time shown on game library badges.",
          options: [
            { label: "Main Story", value: "main" },
            { label: "Main + Extra", value: "extra" },
            { label: "100% Completionist", value: "completionist" },
            { label: "All Styles Average", value: "all" }
          ]
        }
      ]
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    try {
      const res = await fetch("https://howlongtobeat.com");
      if (res.ok) {
        return { success: true, message: "HowLongToBeat connected and operational." };
      }
      return { success: false, message: `HowLongToBeat returned HTTP ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Network error connecting to HowLongToBeat: ${err?.message || err}` };
    }
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      const metric = config?.preferredMetric ?? "main";
      const payload = {
        searchType: "games",
        searchTerms: term.split(" "),
        searchPage: 1,
        size: 1,
        searchOptions: {
          games: {
            userId: 0,
            platform: "",
            sortCategory: "popular",
            rangeCategory: "main",
            rangeTime: { min: null, max: null },
            gameplay: { perspective: "", flow: "", genre: "" },
            rangeYear: { min: null, max: null },
            modifier: ""
          },
          users: { sortCategory: "postcount" },
          lists: { sortCategory: "follows" },
          filter: "",
          sort: 0,
          randomizer: 0
        }
      };

      const res = await fetch("https://howlongtobeat.com/api/search", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Referer: "https://howlongtobeat.com/"
        },
        body: JSON.stringify(payload)
      });
      if (!res.ok) return null;
      const data = await res.json();
      const g = data?.data?.[0];
      if (!g) return null;

      let playtimeHours: number | undefined;
      if (metric === "extra" && g.comp_plus) {
        playtimeHours = Math.round(g.comp_plus / 3600);
      } else if (metric === "completionist" && g.comp_100) {
        playtimeHours = Math.round(g.comp_100 / 3600);
      } else if (g.comp_main) {
        playtimeHours = Math.round(g.comp_main / 3600);
      }

      const imgUrl = g.game_image ? `https://howlongtobeat.com/games/${g.game_image}` : undefined;

      return {
        name: g.game_name || term,
        releaseYear: g.release_world || undefined,
        developers: g.profile_dev ? [g.profile_dev] : undefined,
        playtimeHours,
        capsuleImage: imgUrl,
        heroImage: imgUrl,
        headerImage: imgUrl,
        websiteUrl: g.game_id ? `https://howlongtobeat.com/game/${g.game_id}` : undefined,
        extra: {
          compMainHours: g.comp_main ? Math.round(g.comp_main / 3600) : null,
          compPlusHours: g.comp_plus ? Math.round(g.comp_plus / 3600) : null,
          comp100Hours: g.comp_100 ? Math.round(g.comp_100 / 3600) : null
        }
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
