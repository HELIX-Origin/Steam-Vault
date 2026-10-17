---
layout: docs
title: Plugin Development Guide
summary: Comprehensive guide for authoring platform integrations, metadata providers, and utility plugins using native modules
parent: Documentation Home
nav_order: 14
---

# Plugin Development Guide

Universal Game Launcher provides a native, modular plugin framework (`src/lib/plugins/`). All plugins operate as first-class native modules categorized into dedicated subfolders (`platforms/`, `metadata/`, `utility/`, and `media/`).

Plugins are built directly into the application, ensuring 100% offline functionality, instantaneous loading, non-blocking background execution, and complete immunity to Cloudflare blocks or network rate limits. There are no loose JSON files or external manifests required.

---

## Directory & Category Structure

All plugins are organized by category under `src/lib/plugins/`:

```
src/lib/plugins/
├── framework.ts           # Core UniversalPlugin TypeScript interfaces and contracts
├── engine.ts              # In-memory registry, KV persistence, and connection verification
├── index.ts               # Master registry exporting PLATFORM_PLUGINS, METADATA_PLUGINS, UTILITY_PLUGINS, MEDIA_PLUGINS
├── platforms/             # Platform storefront and game launcher modules
│   ├── steam.ts           # Steam manifest discovery & launch
│   ├── gog.ts             # GOG Galaxy registry detection & direct launch
│   ├── epic.ts            # Epic Games official launcher with optional Legendary CLI support
│   ├── xbox.ts            # PC Game Pass and Windows Store titles
│   ├── itch.ts            # itch.io desktop integration
│   ├── ea.ts              # EA App detection
│   ├── ubisoft.ts         # Ubisoft Connect detection
│   ├── battlenet.ts       # Battle.net client integration
│   ├── amazon.ts          # Amazon Games launcher
│   ├── rockstar.ts        # Rockstar Games Launcher
│   └── standalone.ts      # DRM-free local executables & manual shortcuts
├── metadata/              # Metadata, artwork, and rating scraper modules
│   ├── steam-store.ts     # Valve Steam Store API & CDN artwork
│   ├── steamgriddb.ts     # SteamGridDB grid covers, banners, heroes, logos
│   ├── igdb.ts            # IGDB (Twitch) metadata & synopses
│   ├── itch-store.ts      # itch.io web scraper
│   ├── howlongtobeat.ts   # Community completionist times
│   ├── pcgamingwiki.ts    # Compatibility & PCGW fixes
│   ├── protondb.ts        # Linux & Steam Deck compatibility tiers
│   ├── gog-catalog.ts     # Official GOG product catalog
│   └── epic-store.ts      # Epic Store CDN artwork & descriptions
├── utility/               # Background tools, alerts, and analytics plugins
│   ├── free-games.ts      # Free Game Alerts (Epic, Steam, GOG giveaways)
│   ├── deals.ts           # Store Promotion & Deal Alerts
│   ├── patch-notes.ts     # Patch Notes & Game Updates for installed titles
│   └── storage-analyzer.ts# Storage & Drive Analyzer (non-blocking disk footprint)
└── media/                 # Rich media enhancement plugins
    ├── trailers.ts        # Official video trailer stream player
    └── ambient.ts         # Game audio & soundtrack theme player
```

---

## The `UniversalPlugin` Interface

Every plugin exports a module conforming to `UniversalPlugin` (defined in `src/lib/plugins/framework.ts`):

```ts
export interface UniversalPlugin {
  readonly id: string;
  readonly name: string;
  readonly version: string;
  readonly pluginType: PluginType; // "platform" | "metadata" | "utility" | "media"
  readonly author?: string;
  readonly description?: string;
  readonly icon?: string;
  readonly homepage?: string;
  readonly features?: string[];

  // Dynamic configuration schema rendered in Plugin Settings modal
  getSettingsSchema?(): PluginSettingsConfig;

  // Real-time connectivity or path verification
  testConnection?(config: Record<string, any>): Promise<TestConnectionResult>;

  // Metadata & artwork queries (Metadata plugins)
  queryArtwork?(query: { appId?: number | string; name?: string; platform?: string }, config?: Record<string, any>): Promise<PluginArtworkResult | null>;
  queryMetadata?(query: { appId?: number | string; name?: string; platform?: string }, config?: Record<string, any>): Promise<PluginMetadataResult | null>;

  // Platform launcher integration hooks (Platform plugins)
  getLaunchSpec?(game: { appId?: number | string; appName?: string; exe?: string; launchArgs?: string }, config?: Record<string, any>): PlatformLaunchSpec | null;
  getInstallUri?(game: { appId?: number | string }): string | null;
  getUninstallUri?(game: { appId?: number | string }): string | null;
  getFriendsListUri?(): string | null;
  scanPlatform?(roots?: string[], config?: Record<string, any>): Promise<PlatformScanResult>;
  getDefaultKnownFolders?(): string[];
}
```

---

## Authoring a Platform Plugin

Platform plugins discover, manage, and launch games from specific gaming storefronts or ecosystems.

### Alternate Provider Methods (e.g. Epic Games & Legendary CLI)

A single platform plugin can support multiple launcher methods without needing a separate standalone plugin. For example, the **Epic Games** plugin (`src/lib/plugins/platforms/epic.ts`):
- Defaults to the **official Epic Games Launcher** (`com.epicgames.launcher://`).
- Includes optional support for the open-source **Legendary CLI** backend when the user enables it and specifies the path to their `legendary` binary.

```ts
import type { UniversalPlugin, PlatformLaunchSpec, TestConnectionResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const platformEpicPlugin: UniversalPlugin = {
  id: "platform-epic",
  name: "Epic Games Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "HELIX Origin",
  description: "Discovers and launches games via Epic Games manifests with optional open-source Legendary CLI provider backend.",
  features: ["discovery", "launch", "legendary_sync"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "useLegendary",
          label: "Use Legendary CLI as Alternate Provider Method",
          type: "boolean",
          default: false,
          description: "Use open-source Legendary CLI in place of the official Epic Games Launcher. Requires specifying the path to your legendary binary below."
        },
        {
          key: "customLegendaryPath",
          label: "Custom Legendary CLI Executable Path",
          type: "text",
          description: "Full path to your installed legendary executable (required if Legendary provider is enabled).",
          placeholder: "C:\\path\\to\\legendary.exe"
        }
      ]
    };
  },

  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    const customPath = config?.customLegendaryPath?.trim();
    const useLegendary = config?.useLegendary === true && Boolean(customPath);

    // Alternate Provider: Legendary CLI
    if (useLegendary && customPath) {
      return {
        executablePath: customPath,
        args: ["launch", game.appName || game.appId.toString()]
      };
    }

    // Default: Official Epic Games Launcher
    const appId = game.appName || game.appId;
    if (appId) {
      return {
        protocolUri: `com.epicgames.launcher://apps/${appId}?action=launch&silent=true`
      };
    }

    return null;
  }
};
```

---

## Authoring a Utility Plugin

Utility plugins provide background tools, alerts, drive analysis, and notification feeds.

### Example: Free Game Alerts Plugin (`src/lib/plugins/utility/free-games.ts`)

```ts
import type { UniversalPlugin } from "../framework";

export const freeGameAlertsPlugin: UniversalPlugin = {
  id: "utility-free-games",
  name: "Free Game Alerts",
  version: "1.0.0",
  pluginType: "utility",
  author: "HELIX-Origin",
  description: "Monitors and notifies of 100% free store giveaways and promotional offers across Epic Games, Steam, and GOG.",
  homepage: "https://github.com/HELIX-Origin/Universal-Game-Launcher",
  features: ["notifications", "giveaways", "free_games"],
  getSettingsSchema: () => ({
    requiresApiKey: false,
    fields: [
      {
        key: "pollIntervalMinutes",
        label: "Check Interval (Minutes)",
        type: "number",
        default: 60,
        description: "How often to query storefront giveaway APIs for newly announced free titles."
      }
    ]
  })
};
```

### Example: Storage & Drive Analyzer Plugin (`src/lib/plugins/utility/storage-analyzer.ts`)

```ts
import type { UniversalPlugin } from "../framework";

export const storageAnalyzerPlugin: UniversalPlugin = {
  id: "utility-storage",
  name: "Storage & Drive Analyzer",
  version: "1.0.0",
  pluginType: "utility",
  author: "HELIX-Origin",
  description: "Calculates disk footprint across all platform directories and library drives using non-blocking background workers.",
  features: ["disk_analysis", "drive_reporting", "non_blocking_worker"]
};
```

---

## Authoring a Metadata Provider Plugin

Metadata provider plugins enrich titles with descriptions, genres, user ratings, and CDN artwork.

### Example: Custom Metadata Provider (`src/lib/plugins/metadata/my-provider.ts`)

```ts
import type { UniversalPlugin, PluginMetadataResult, PluginArtworkResult, TestConnectionResult } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const myProviderPlugin: UniversalPlugin = {
  id: "metadata-my-provider",
  name: "My Metadata Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "Community Contributor",
  description: "Enriches titles with custom artwork and reviews from My Metadata Service.",

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: true,
      fields: [
        {
          key: "apiKey",
          label: "API Key",
          type: "password",
          required: true,
          description: "API key from https://mymetadata.example.com/account"
        }
      ]
    };
  },

  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    if (!config?.apiKey) {
      return { success: false, message: "Missing API Key." };
    }
    return { success: true, message: "Connection verified." };
  },

  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    return {
      capsuleImage: "https://cdn.example.com/capsule.jpg",
      heroImage: "https://cdn.example.com/hero.jpg",
      logoImage: "https://cdn.example.com/logo.png"
    };
  },

  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    return {
      name: query.name,
      shortDescription: "Exciting adventure game.",
      genres: ["Action", "Adventure"],
      rating: 8.5
    };
  }
};
```

---

## Registering Plugins

1. Export your plugin module from its category index:
   - Platform plugins: `src/lib/plugins/platforms/index.ts`
   - Metadata plugins: `src/lib/plugins/metadata/index.ts`
   - Utility plugins: `src/lib/plugins/utility/index.ts`
   - Media plugins: `src/lib/plugins/media/index.ts`
2. Add your plugin instance to `ALL_PLUGINS` in `src/lib/plugins/index.ts`.
3. If the plugin provides backend Rust hooks, register its manifest in `src-tauri/src/plugins/defaults.rs` under `default_plugin_manifests()`.
4. Run quality gates to verify clean compilation:
   ```powershell
   npm run check
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
