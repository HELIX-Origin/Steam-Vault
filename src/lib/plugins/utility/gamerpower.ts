import type { UniversalPlugin } from "../framework";

export const gamerpowerPlugin: UniversalPlugin = {
  id: "utility-gamerpower",
  name: "GamerPower Universal Feeds & Alerts",
  version: "1.0.0",
  pluginType: "utility",
  author: "HELIX-Origin",
  description: "Universal aggregator for 100% free game giveaways, verified store promotions & deals, and game patch notes across Steam, Epic Games, GOG, itch.io, Xbox, and more via the official GamerPower API.",
  homepage: "https://www.gamerpower.com",
  features: ["notifications", "giveaways", "free_games", "store_deals", "promotions", "patch_notes", "game_updates"],
  getSettingsSchema: () => ({
    requiresApiKey: false,
    fields: [
      {
        key: "enableFreeGames",
        label: "100% Free Game Alerts",
        type: "boolean",
        default: true,
        description: "Monitor and display 100% free game giveaways and limited-time promotions across all supported platforms."
      },
      {
        key: "enableDeals",
        label: "Store Deals & Promotions",
        type: "boolean",
        default: true,
        description: "Aggregate highlighted game sales, special offers, and major publisher discounts."
      },
      {
        key: "enablePatchNotes",
        label: "Patch Notes & Game Updates",
        type: "boolean",
        default: true,
        description: "Fetch official developer update news, community announcements, and patch changelogs for installed games."
      },
      {
        key: "minDiscountPercent",
        label: "Minimum Discount %",
        type: "number",
        default: 50,
        description: "Only show deals with a discount percentage equal to or greater than this threshold."
      },
      {
        key: "pollIntervalMinutes",
        label: "Check Interval (Minutes)",
        type: "number",
        default: 60,
        description: "How often to query feed APIs in the background for newly announced free titles and deals."
      },
      {
        key: "notifyDesktop",
        label: "Desktop Push Notification",
        type: "boolean",
        default: true,
        description: "Display an operating system notification when a new free game becomes claimable."
      }
    ]
  })
};
