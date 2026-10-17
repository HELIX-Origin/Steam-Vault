---
layout: wiki
title: Plugins Architecture & Ecosystem
summary: Deep dive into the Universal Game Launcher native modular plugins architecture, category subfolders, and non-blocking background workers
parent: Technical Wiki
nav_order: 9
---

# Plugins Architecture & Ecosystem

Universal Game Launcher implements an in-app native modular plugins framework (`src/lib/plugins/`) designed for instantaneous game discovery, multi-provider metadata enrichment, utility alert feeds, and full launcher capability integration with zero external network failure points or loose JSON manifest files.

---

## Architectural Principles

1. **Native In-App Modular Framework**: All plugins live directly within the application codebase under `src/lib/plugins/` (subdivided into `platforms/`, `metadata/`, `utility/`, and `media/`). This completely eliminates Cloudflare network blockages, CDN rate limits, runtime `.mjs` eval vulnerabilities, and fragile external JSON manifests.
2. **Dedicated Per-Plugin Architecture**: Every platform, metadata provider, and utility feature is authored as its own dedicated, isolated file.
3. **Category Subfolder Organization**:
   - `platforms/`: Storefront integrations (Steam, GOG, Epic Games, Xbox, EA, Ubisoft, Battle.net, itch.io, Amazon Games, Rockstar Games, Standalone, EmulationStation).
   - `metadata/`: Metadata, artwork, and rating scrapers (Steam Store, GOG Catalog, Epic Games Store, itch.io Store, SteamGridDB, IGDB, TheGamesDB, ScreenScraper, HowLongToBeat, PCGamingWiki, ProtonDB).
   - `utility/`: Background tools, free game alerts, deal tracking, patch notes, and drive analyzers (`utility-gamerpower`, `utility-storage`).
   - `media/`: Media enhancements (video trailers, ambient audio).
4. **Platform Providers with Alternate Launcher Backends**: Rather than duplicating platform code across detached plugins, platform plugins natively support optional alternate methods. For example, the **Epic Games** plugin defaults to the official Epic Games Launcher (`com.epicgames.launcher://`), while seamlessly providing an optional toggle to use the open-source **Legendary CLI** engine when the user provides their custom binary path.
5. **Universal Non-Blocking Background Workers**: Heavy filesystem operations (such as drive space scanning in `utility-storage`) and external API queries (such as `utility-gamerpower`) run asynchronously via `tauri::async_runtime::spawn_blocking` and Svelte background tasks. Switching tabs never freezes or halts the user interface; UI structures render instantly while values populate reactively.
6. **Reactive Plugin-Aware Tab Filtering**:
   - **Dashboard**: Feeds, notifications, quick launch cards, and library stats reactively filter based on active plugins. If `utility-gamerpower` is disabled, free game giveaways and deals are hidden or muted.
   - **Storage Tab**: Drive footprint tables and breakdown cards only display information for enabled platform plugins and when `utility-storage` is active.
7. **Tabbed Plugin Settings Sub-Window**: An interactive settings modal renders dedicated configuration tabs for each enabled plugin declaring settings, complete with live credential testing and password visibility toggles.
8. **Air-Gapped Local Operation**: The application functions 100% offline out of the box.

```
┌────────────────────────────────────────────────────────┐
│               Universal Game Launcher                  │
│                                                        │
│  ┌───────────────────────────┐  ┌───────────────────┐  │
│  │     SvelteKit Frontend    │─▶│ In-Memory Redis KV│  │
│  │   src/lib/plugins/engine  │  │      mem_kv       │  │
│  └─────────────┬─────────────┘  └─────────┬─────────┘  │
│                │ Direct Module Imports    │            │
│                ▼                          ▼            │
│  ┌───────────────────────────┐  ┌───────────────────┐  │
│  │ Dedicated Plugin Modules  │  │   SQLite Database │  │
│  │  platforms/*, metadata/*  │  │  database.sqlite  │  │
│  │  utility/*, media/*       │  │                   │  │
│  └───────────────────────────┘  └───────────────────┘  │
└────────────────────────────────────────────────────────┘
```

---

## The `UniversalPlugin` Contract (`framework.ts`)

Every plugin module exports an object conforming to `UniversalPlugin`:

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

  // Dynamic platform metadata for platform plugins
  readonly platformId?: string;
  readonly platformLabel?: string;
  readonly platformColor?: string;
  readonly defaultStoreUrl?: string;

  // Dynamic configuration schema
  getSettingsSchema?(): PluginSettingsConfig;

  // Real-time connectivity or path verification
  testConnection?(config: Record<string, any>): Promise<TestConnectionResult>;

  // Metadata & artwork retrieval
  queryArtwork?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginArtworkResult | null>;
  queryMetadata?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginMetadataResult | null>;

  // Platform launcher integration hooks
  detectLauncher?(config?: Record<string, any>): Promise<LauncherDetectionResult>;
  getLaunchSpec?(
    game: { appId?: number | string; appName?: string; installPath?: string; exe?: string; launchArgs?: string },
    config?: Record<string, any>
  ): PlatformLaunchSpec | null;
  getInstallUri?(game: { appId?: number | string; appName?: string }): string | null;
  getUninstallUri?(game: { appId?: number | string; appName?: string }): string | null;
  getStorePageUri?(game: { appId?: number | string; appName?: string }): string | null;

  // Social & Friends list hooks
  getFriendsListUri?(): string | null;
  getFriendsList?(config?: Record<string, any>): Promise<PlatformFriend[] | null>;

  // Achievements tracking hooks
  getAchievementsUri?(game: { appId?: number | string; appName?: string }): string | null;
  getAchievements?(game: { appId?: number | string; appName?: string }, config?: Record<string, any>): Promise<PlatformAchievement[] | null>;

  // Game scanning & folder hooks
  scanPlatform?(roots?: string[], config?: Record<string, any>): Promise<PlatformScanResult>;
  getDefaultKnownFolders?(): string[];
}
```

---

## Metadata Resolution & The Cache Reset Invariant

Universal Game Launcher enforces an authoritative resolution hierarchy:
1. **Initial Acquisition**: Platform plugins provide authoritative default metadata and artwork directly from the platform's official client or storefront.
2. **On-Demand Enhancement**: When a user clicks **✨ Enhance Metadata**, third-party metadata plugins enrich the record. By default, only missing fields are filled.
3. **Protection**: Enhanced records are flagged with `userEnhanced: true` and cached under `game_meta:<appId>`. Normal background scans will **never** overwrite user-enhanced fields.
4. **Cache Reset**: Game providers are strictly permitted to overwrite metadata provider contributions only when the user explicitly triggers **↺ Reset Cache to Provider Defaults**.

---

## Utility Plugins & Feed Integration

Utility plugins reside in `src/lib/plugins/utility/` and supply dynamic application-level services:

| Plugin ID | Name | Description |
| :--- | :--- | :--- |
| `utility-gamerpower` | GamerPower Feeds & Alerts | Universal aggregator for 100% free store giveaways, verified store promotions & deals, and game patch notes across Steam, Epic, GOG, itch, and Xbox. |
| `utility-storage` | Storage & Drive Analyzer | Performs non-blocking drive capacity analysis and platform game footprints via background worker threads. |

### UI Tab Reactivity

Components check plugin status via reactive store derived values:

```ts
function isPluginEnabled(pluginId: string): boolean {
  if ($settings?.enabledPlugins && $settings.enabledPlugins[pluginId] !== undefined) {
    return $settings.enabledPlugins[pluginId];
  }
  return true;
}
```

When a user toggles a utility plugin off in **Settings → Plugins**:
1. Active notification count badges update instantly without a page reload.
2. Background fetch tasks for disabled feeds are bypassed, saving network bandwidth.
3. Relevant dashboard tabs gracefully display helpful status notices with a one-click link to Settings.

---

## Platform Providers & Alternate Launch Backends

Platform plugins (`src/lib/plugins/platforms/`) support flexible launch delegation:

### Epic Games Integration (`platform-epic`)
- **Default Method**: Official Epic Games Launcher via protocol URI `com.epicgames.launcher://apps/{appId}?action=launch&silent=true`.
- **Alternate Method (Legendary CLI)**: When enabled in plugin settings (`useLegendary: true`) along with a valid executable path (`customLegendaryPath`), games are executed headlessly via `legendary launch <appName>` with automatic cloud save synchronization.
- **Epic Online Services Overlay**: Optional toggle for EOS overlay injection.

### EmulationStation Integration (`platform-emulationstation`)
- **Multi-Console Directory Scanning**: Scans ROMs across standard console subdirectories (`nes`, `snes`, `n64`, `gc`, `wii`, `psx`, `ps2`, etc.).
- **Automatic Scaffolding**: Can generate standard folder directories populated with `systeminfo.txt`.
- **Intelligent Title Normalization**: Strips dump tags `(USA)`, `[!]`, `(Rev 1)`, GoodTools codes, and NKit compound extensions to display clean game titles.

---

## ➡️ Next Steps

- [Plugin Developer Guide](Plugin-Developer-Guide.md) — complete step-by-step developer tutorial with templates.
- [Available Plugins Catalog]({{ '/docs/Available-Plugins/' | relative_url }}) — explore all 27 built-in plugins.
- [Architecture](Architecture.md) — overall system architecture.
