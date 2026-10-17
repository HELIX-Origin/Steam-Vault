---
layout: wiki
title: Plugin Developer Guide
summary: Complete step-by-step developer tutorial for authoring platform, metadata, utility, and media plugins
parent: Technical Wiki
nav_order: 10
---

# Plugin Developer Guide

This developer guide walks through creating new plugins for **Universal Game Launcher**.

Universal Game Launcher uses an **in-app native modular plugin architecture** (`src/lib/plugins/`). All plugins operate as first-class TypeScript modules compiled directly into the application, eliminating loose JSON files, network CDN vulnerabilities, and runtime script injection risks.

---

## 🏗️ The Plugin Framework Overview

All plugins reside in `src/lib/plugins/` under four categorical subdirectories:
- `src/lib/plugins/platforms/` — Game storefronts, client launchers, local executables, and emulator platforms.
- `src/lib/plugins/metadata/` — Third-party metadata, community box art, and compatibility providers.
- `src/lib/plugins/utility/` — Background notification feeds, giveaway scrapers, and disk storage analyzers.
- `src/lib/plugins/media/` — Video trailer streams, hero background animations, and ambient game soundtracks.

Every plugin exports a constant implementing the `UniversalPlugin` interface defined in `src/lib/plugins/framework.ts`.

---

## Plugin Implementation Guides

Step-by-step guides and copy-pasteable TypeScript templates for each of the four plugin categories.

### Platform Integration Plugins

Platform plugins discover installed titles, formulate launch commands, query native client details, and connect to social and achievement hooks.

#### Required Hooks & Capabilities
1. **Metadata & Identity**: Unique `id` (prefixed with `platform-`), `name`, `version`, `pluginType: "platform"`.
2. **Platform Attributes**: `platformId`, `platformLabel`, `platformColor`, `defaultStoreUrl`.
3. **Launch Spec (`getLaunchSpec`)**: Returns either a `protocolUri` (e.g. `steam://rungameid/{id}`) or an `executablePath` with `args`.
4. **Native Metadata & Artwork (`queryMetadata`, `queryArtwork`)**: Queries official storefront endpoints directly to provide authoritative game information.
5. **Social & Achievements**: Returns URI schemes or records for live friends lists and achievements.
6. **Configurable Settings (`getSettingsSchema`)**: Exposes custom binary paths, alternate launch backends (e.g. CLI tools), or multi-drive options.

#### Complete Platform Plugin Template

Below is a complete, copy-pasteable platform plugin implementation (`src/lib/plugins/platforms/my-platform.ts`):

```ts
import type {
  UniversalPlugin,
  PlatformLaunchSpec,
  PlatformScanResult,
  LauncherDetectionResult,
  TestConnectionResult,
  PluginMetadataResult,
  PluginArtworkResult,
  PlatformFriend,
  PlatformAchievement
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const platformMyPlatformPlugin: UniversalPlugin = {
  id: "platform-myplatform",
  name: "MyPlatform Integration",
  version: "1.0.0",
  pluginType: "platform",
  author: "Community Contributor",
  description: "Discovers and launches games from MyPlatform client installations and official protocol URIs.",
  homepage: "https://myplatform.example.com",
  features: [
    "discovery",
    "launch",
    "myplatform_protocol",
    "social",
    "achievements",
    "store_cdn"
  ],

  // Visual metadata for library UI
  platformId: "myplatform",
  platformLabel: "MyPlatform",
  platformColor: "#6366f1",
  defaultStoreUrl: "https://myplatform.example.com/store",

  // Settings schema rendered in the Tabbed Plugin Settings Dialog
  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "customClientPath",
          label: "Custom Client Executable Path",
          type: "text",
          description: "Optional custom path to MyPlatformClient.exe if installed outside standard paths.",
          placeholder: "C:\\Program Files\\MyPlatform\\client.exe"
        },
        {
          key: "enableOverlay",
          label: "Enable In-Game Overlay",
          type: "boolean",
          default: true,
          description: "Inject the official MyPlatform client overlay when launching games."
        }
      ]
    };
  },

  // Real-time verification in Plugin Settings dialog
  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const customPath = config?.customClientPath?.trim();
    if (customPath) {
      return { success: true, message: `Custom executable configured: ${customPath}` };
    }
    return { success: true, message: "Standard client protocol discovery active." };
  },

  // Launcher detection
  async detectLauncher(config?: Record<string, any>): Promise<LauncherDetectionResult> {
    return {
      installed: true,
      method: "system_store",
      message: "MyPlatform client detected via system registry."
    };
  },

  // Launch specification: protocol or direct binary
  getLaunchSpec(game, config): PlatformLaunchSpec | null {
    const appId = game.appName || game.appId;
    if (!appId) return null;

    // Delegate launch to native client protocol
    return {
      protocolUri: `myplatform://launch/${appId}?overlay=${config?.enableOverlay !== false}`
    };
  },

  // Install & Uninstall protocols
  getInstallUri(game): string | null {
    return game.appId ? `myplatform://install/${game.appId}` : null;
  },
  getUninstallUri(game): string | null {
    return game.appId ? `myplatform://uninstall/${game.appId}` : null;
  },
  getStorePageUri(game): string | null {
    return game.appId ? `https://myplatform.example.com/app/${game.appId}` : null;
  },

  // Social & Friends list integration
  getFriendsListUri(): string | null {
    return "myplatform://friends";
  },
  async getFriendsList(config): Promise<PlatformFriend[] | null> {
    // In a real implementation, inspect client local state or API
    return [
      {
        id: "friend_1",
        name: "GamerOne",
        status: "in_game",
        gameTitle: "Space Adventure",
        avatarUrl: null
      }
    ];
  },

  // Achievements integration
  getAchievementsUri(game): string | null {
    return game.appId ? `myplatform://achievements/${game.appId}` : null;
  },
  async getAchievements(game, config): Promise<PlatformAchievement[] | null> {
    return null;
  },

  // Native authoritative metadata fetching
  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    if (!query.appId && !query.name) return null;

    try {
      // Fetch authoritative details from official storefront API
      const res = await fetch(`https://api.myplatform.example.com/games/${query.appId}`);
      if (!res.ok) return null;
      const data = await res.json();

      return {
        name: data.title,
        shortDescription: data.summary,
        releaseDate: data.release_date,
        releaseYear: data.release_year,
        developers: data.developers || [],
        publishers: data.publishers || [],
        genres: data.genres || [],
        rating: data.score
      };
    } catch {
      return null;
    }
  },

  // Native authoritative artwork fetching
  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    if (!query.appId) return null;

    return {
      capsuleImage: `https://cdn.myplatform.example.com/capsules/${query.appId}.jpg`,
      heroImage: `https://cdn.myplatform.example.com/heroes/${query.appId}.jpg`,
      logoImage: `https://cdn.myplatform.example.com/logos/${query.appId}.png`
    };
  }
};
```

---

### Metadata Provider Plugins

Metadata provider plugins enrich titles with descriptions, community covers, genres, and completion statistics. They are invoked when a user clicks **✨ Enhance Metadata** in the Library showcase view.

#### The Role Hierarchy & Invariant Contract
- **Non-Destructive by Default**: By default, `enhanceGameMetadata` runs in "fill missing fields only" mode. It populates empty descriptions or missing artwork without replacing existing data.
- **Override Mode**: If the user checks the **Override Existing Metadata** box, all fields returned by your plugin overwrite existing properties.
- **Protection**: Once enhanced, records are marked with `userEnhanced: true`. Native game provider scans will never overwrite metadata provider contributions unless the user explicitly clicks **↺ Reset Cache to Provider Defaults**.

#### Complete Metadata Plugin Template

Below is a complete, copy-pasteable metadata plugin implementation (`src/lib/plugins/metadata/my-metadata.ts`):

```ts
import type {
  UniversalPlugin,
  PluginMetadataResult,
  PluginArtworkResult,
  TestConnectionResult
} from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const myMetadataPlugin: UniversalPlugin = {
  id: "metadata-mymetadata",
  name: "MyMetadata Database Provider",
  version: "1.0.0",
  pluginType: "metadata",
  author: "Community Contributor",
  description: "Enriches game records with community reviews, genres, and 4K box art from MyMetadata API.",
  homepage: "https://mymetadata.example.com",
  features: [
    "game_metadata",
    "boxart",
    "developer_credits",
    "critic_scores"
  ],

  // Expose configuration in the Tabbed Plugin Settings Dialog
  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: true,
      fields: [
        {
          key: "apiKey",
          label: "API Access Key",
          type: "password",
          required: true,
          description: "Your free API key from https://mymetadata.example.com/developer"
        },
        {
          key: "preferredLanguage",
          label: "Preferred Metadata Language",
          type: "select",
          default: "en",
          options: [
            { label: "English", value: "en" },
            { label: "German", value: "de" },
            { label: "French", value: "fr" },
            { label: "Japanese", value: "ja" }
          ],
          description: "Language to request for game descriptions and titles."
        }
      ]
    };
  },

  // Connection testing in Plugin Settings dialog
  async testConnection(config: Record<string, any>): Promise<TestConnectionResult> {
    const key = config?.apiKey?.trim();
    if (!key) {
      return { success: false, message: "API key is required." };
    }

    try {
      const res = await fetch("https://api.mymetadata.example.com/v1/ping", {
        headers: { Authorization: `Bearer ${key}` }
      });
      if (res.ok) {
        return { success: true, message: "Successfully connected to MyMetadata API." };
      }
      return { success: false, message: `Server returned HTTP status ${res.status}.` };
    } catch (e: any) {
      return { success: false, message: `Connection failed: ${e.message}` };
    }
  },

  // Querying descriptive metadata
  async queryMetadata(query, config): Promise<PluginMetadataResult | null> {
    const title = query.name?.trim();
    if (!title) return null;

    const apiKey = config?.apiKey?.trim();
    const lang = config?.preferredLanguage || "en";

    try {
      const url = `https://api.mymetadata.example.com/v1/search?q=${encodeURIComponent(title)}&lang=${lang}`;
      const res = await fetch(url, {
        headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {}
      });
      if (!res.ok) return null;

      const data = await res.json();
      const match = data.results?.[0];
      if (!match) return null;

      return {
        name: match.name,
        shortDescription: match.synopsis,
        detailedDescription: match.description,
        releaseDate: match.release_date,
        releaseYear: match.release_year ? Number(match.release_year) : undefined,
        developers: match.developers || [],
        publishers: match.publishers || [],
        genres: match.genres || [],
        tags: match.tags || [],
        rating: match.score,
        userScore: match.user_rating,
        criticScore: match.critic_score,
        players: match.multiplayer_support ? "Multiplayer" : "Single-player"
      };
    } catch {
      return null;
    }
  },

  // Querying box art and promotional images
  async queryArtwork(query, config): Promise<PluginArtworkResult | null> {
    const title = query.name?.trim();
    if (!title) return null;

    const apiKey = config?.apiKey?.trim();

    try {
      const res = await fetch(`https://api.mymetadata.example.com/v1/artwork?q=${encodeURIComponent(title)}`, {
        headers: apiKey ? { Authorization: `Bearer ${apiKey}` } : {}
      });
      if (!res.ok) return null;

      const art = await res.json();
      return {
        capsuleImage: art.cover_url || null,
        heroImage: art.banner_url || null,
        logoImage: art.logo_url || null,
        screenshots: art.screenshots || []
      };
    } catch {
      return null;
    }
  }
};
```

---

### Utility Plugins

Utility plugins provide background tools, live giveaway notifications, and disk analyzers. They are executed asynchronously and must never freeze the UI.

#### Complete Utility Plugin Template (`src/lib/plugins/utility/my-utility.ts`):

```ts
import type { UniversalPlugin } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const myUtilityPlugin: UniversalPlugin = {
  id: "utility-mytool",
  name: "My Custom Utility Tool",
  version: "1.0.0",
  pluginType: "utility",
  author: "Community Contributor",
  description: "Monitors game community news and notifies of upcoming events.",
  features: ["news_feed", "notifications", "background_worker"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "checkIntervalHours",
          label: "Update Interval (Hours)",
          type: "number",
          default: 12,
          description: "How frequently to query the background news feed."
        },
        {
          key: "notifyOnImportant",
          label: "Show Desktop Notifications",
          type: "boolean",
          default: true,
          description: "Show a native desktop toast notification when high-priority news arrives."
        }
      ]
    };
  }
};
```

---

### Media Enhancement Plugins

Media plugins provide multimedia additions such as hero video trailers and background ambient music.

#### Complete Media Plugin Template (`src/lib/plugins/media/my-media.ts`):

```ts
import type { UniversalPlugin } from "../framework";
import type { PluginSettingsConfig } from "$lib/types";

export const myMediaPlugin: UniversalPlugin = {
  id: "media-mymedia",
  name: "My Media Streamer",
  version: "1.0.0",
  pluginType: "media",
  author: "Community Contributor",
  description: "Streams community walkthrough previews in the game showcase view.",
  features: ["video_playback", "walkthroughs"],

  getSettingsSchema(): PluginSettingsConfig {
    return {
      requiresApiKey: false,
      fields: [
        {
          key: "autoPreview",
          label: "Auto-Play Video Previews",
          type: "boolean",
          default: false,
          description: "Automatically stream game previews on selection."
        }
      ]
    };
  }
};
```

---

## 📦 Registering & Testing Your Plugin

Once your plugin file is authored in its category directory, complete registration with three simple steps:

### 1. Register in the Master Catalog (`src/lib/plugins/catalog.ts`)
Import your plugin and add it to its category array:

```ts
// For a Platform Plugin:
import { platformMyPlatformPlugin } from "./platforms/my-platform";
export const PLATFORM_PLUGINS: UniversalPlugin[] = [
  // ... existing platforms
  platformMyPlatformPlugin
];

// For a Metadata Plugin:
import { myMetadataPlugin } from "./metadata/my-metadata";
export const METADATA_PLUGINS: UniversalPlugin[] = [
  // ... existing metadata providers
  myMetadataPlugin
];
```

The master `ALL_PLUGINS` array automatically incorporates your plugin.

### 2. Register Default State in Rust (`src-tauri/src/plugins/defaults.rs`)
If your plugin provides backend platform inspection or default enabled state, add an entry to `default_plugin_manifests()`:

```rust
PluginManifest {
    id: "platform-myplatform".to_string(),
    name: "MyPlatform Integration".to_string(),
    version: "1.0.0".to_string(),
    plugin_type: "platform".to_string(),
    author: "Community Contributor".to_string(),
    description: "Discovers and launches games from MyPlatform.".to_string(),
    enabled: true,
    requires_api_key: false,
    ..Default::default()
}
```

### 3. Run Quality Verification Gates
Verify that TypeScript compilation and Rust checks pass cleanly:

```powershell
# Frontend TypeScript & Svelte check
npm run check

# Rust backend compiler check
cargo check --manifest-path src-tauri/Cargo.toml
```

---

## ➡️ Next Steps

- [Plugins Architecture & Ecosystem](Plugins-Architecture.md) — architectural deep-dive into non-blocking workers and KV caching.
- [Available Plugins Catalog]({{ '/docs/Available-Plugins/' | relative_url }}) — explore the 27 built-in plugins.
- [Metadata Providers]({{ '/docs/Metadata-Providers/' | relative_url }}) — role hierarchy and cache reset invariant.
