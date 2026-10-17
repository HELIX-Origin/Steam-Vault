import type { UniversalPlugin, TestConnectionResult, PluginMetadataResult, PluginArtworkResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const pcgamingwikiPlugin: UniversalPlugin = {
  id: "metadata-pcgamingwiki",
  name: "PCGamingWiki Fixes & Compatibility",
  version: "1.0.0",
  pluginType: "metadata",
  author: "HELIX Origin",
  description: "Supplies controller support flags, ultra-wide FOV fixes, cloud save locations, and community fixes from PCGamingWiki.",
  homepage: "https://www.pcgamingwiki.com",
  features: ["compatibility", "fov_fixes", "controller_support", "save_locations"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "showFixesInDetails",
          label: "Display Bugfixes in Game Details",
          type: "boolean",
          default: true,
          description: "Show notable configuration tweaks and bugfixes on game showcase pages."
        }
      ]
    };
  },

  async testConnection(): Promise<TestConnectionResult> {
    try {
      const res = await fetch("https://www.pcgamingwiki.com/w/api.php?action=query&meta=siteinfo&format=json");
      if (res.ok) {
        return { success: true, message: "PCGamingWiki Cargo API connected and operational without key." };
      }
      return { success: false, message: `PCGamingWiki returned HTTP ${res.status}.` };
    } catch (err: any) {
      return { success: false, message: `Failed to connect to PCGamingWiki: ${err?.message || err}` };
    }
  },

  async queryMetadata(query): Promise<PluginMetadataResult | null> {
    const term = query.name?.trim();
    if (!term) return null;

    try {
      // 1. Search for page
      const searchRes = await fetch(`https://www.pcgamingwiki.com/w/api.php?action=opensearch&search=${encodeURIComponent(term)}&limit=1&format=json`);
      if (!searchRes.ok) return null;
      const searchData = await searchRes.json();
      const pageTitle = searchData?.[1]?.[0];
      const pageUrl = searchData?.[3]?.[0];
      if (!pageTitle) return null;

      // 2. Query Cargo table for infobox data
      const cargoUrl = `https://www.pcgamingwiki.com/w/api.php?action=cargoquery&tables=Infobox_game&fields=Infobox_game._pageName=Page,Infobox_game.Developers,Infobox_game.Publishers,Infobox_game.Released,Infobox_game.Cover_URL&where=Infobox_game._pageName=%22${encodeURIComponent(pageTitle.replace(/"/g, ''))}%22&format=json`;
      const cargoRes = await fetch(cargoUrl);
      if (!cargoRes.ok) return null;
      const cargoData = await cargoRes.json();
      const row = cargoData?.cargoquery?.[0]?.title;

      const devs = row?.Developers ? (row.Developers as string).split(",").map(s => s.trim()).filter(Boolean) : undefined;
      const pubs = row?.Publishers ? (row.Publishers as string).split(",").map(s => s.trim()).filter(Boolean) : undefined;
      const releaseDate = row?.Released || undefined;
      const releaseYear = releaseDate ? parseInt(releaseDate.slice(0, 4), 10) || undefined : undefined;
      const coverUrl = row?.Cover_URL ? (row.Cover_URL.startsWith("http") ? row.Cover_URL : `https://www.pcgamingwiki.com/wiki/${row.Cover_URL}`) : undefined;

      return {
        name: pageTitle,
        releaseDate,
        releaseYear,
        developers: devs,
        publishers: pubs,
        capsuleImage: coverUrl,
        heroImage: coverUrl,
        headerImage: coverUrl,
        websiteUrl: pageUrl || `https://www.pcgamingwiki.com/wiki/${encodeURIComponent(pageTitle)}`
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
