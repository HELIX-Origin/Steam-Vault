---
layout: docs
title: Plugin Development
summary: Architecture and implementation guide for authoring native in-app modular plugins
parent: Documentation Home
nav_order: 14
---

# Plugin Development

Universal Game Launcher provides a native, modular plugin framework (`src/lib/plugins/`). All plugins operate as first-class native TypeScript modules categorized into dedicated subfolders (`platforms/`, `metadata/`, `utility/`, and `media/`).

Plugins are built directly into the application, ensuring 100% offline functionality, instantaneous loading, non-blocking background execution, and complete immunity to Cloudflare blocks or network rate limits. There are no loose JSON files or external manifests required.

---

## Directory & Category Structure

All plugins are organized by category under `src/lib/plugins/`:

```
src/lib/plugins/
├── framework.ts           # Core UniversalPlugin TypeScript interfaces and contracts
├── engine.ts              # In-memory registry, KV persistence, metadata enhancement & reset
├── catalog.ts             # Master catalog exporting PLATFORM_PLUGINS, METADATA_PLUGINS, etc.
├── index.ts               # Public entrypoint
├── platforms/             # Platform storefront and game launcher modules (12 plugins)
│   ├── steam.ts           # Steam manifest discovery, launch, social, achievements & store API
│   ├── gog.ts             # GOG Galaxy registry detection, direct launch & catalog
│   ├── epic.ts            # Epic Games official launcher with optional Legendary CLI support
│   ├── xbox.ts            # PC Game Pass and Windows Store titles
│   ├── itch.ts            # itch.io desktop integration & indie catalog
│   ├── ea.ts              # EA Desktop detection, launch & metadata
│   ├── ubisoft.ts         # Ubisoft Connect detection & launch
│   ├── battlenet.ts       # Battle.net client integration & launch
│   ├── amazon.ts          # Amazon Games launcher integration
│   ├── rockstar.ts        # Rockstar Games Launcher integration
│   ├── standalone.ts      # DRM-free local executables & manual shortcuts
│   └── emulationstation.ts# Multi-console ROM scanning & title normalization
├── metadata/              # Metadata, artwork, and rating scraper modules (11 plugins)
│   ├── steam-store.ts     # Valve Steam Store API & CDN artwork
│   ├── steamgriddb.ts     # SteamGridDB grid covers, banners, heroes, logos
│   ├── igdb.ts            # IGDB (Twitch) metadata, taxonomy & synopses
│   ├── thegamesdb.ts      # TheGamesDB retro ROM box art & fanart
│   ├── screenscraper.ts   # ScreenScraper 2D/3D box art, wheel logos, video snaps
│   ├── itch-store.ts      # itch.io store catalog & indie creator metadata
│   ├── howlongtobeat.ts   # Community completionist times
│   ├── pcgamingwiki.ts    # Compatibility & PCGW fixes
│   ├── protondb.ts        # Linux & Steam Deck compatibility tiers
│   ├── gog-catalog.ts     # Official GOG product catalog
│   └── epic-store.ts      # Epic Store CDN artwork & descriptions
├── utility/               # Background tools, alerts, and analytics plugins (2 plugins)
│   ├── gamerpower.ts      # GamerPower Universal Feeds (Free Games, Deals & Patch Notes)
│   └── storage-analyzer.ts# Storage & Drive Analyzer (non-blocking disk footprint)
└── media/                 # Rich media enhancement plugins (2 plugins)
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

  // Dynamic platform metadata (for platform plugins)
  readonly platformId?: string;
  readonly platformLabel?: string;
  readonly platformColor?: string;
  readonly defaultStoreUrl?: string;

  // Dynamic configuration schema rendered in Plugin Settings modal
  getSettingsSchema?(): PluginSettingsConfig;

  // Real-time connectivity or path verification
  testConnection?(config: Record<string, any>): Promise<TestConnectionResult>;

  // Metadata & artwork queries (Both Metadata plugins and Platform plugins)
  queryArtwork?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginArtworkResult | null>;
  queryMetadata?(
    query: { appId?: number | string; name?: string; platform?: string; platformId?: string },
    config?: Record<string, any>
  ): Promise<PluginMetadataResult | null>;

  // Platform launcher integration hooks (Platform plugins)
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

## Authoring Plugins by Category

### 1. Platform Plugins
Platform plugins discover games, formulate launch specs, query native metadata, and hook into friends/achievements.
- See the [Epic Games Integration (`platform-epic`)](Available-Plugins.md#2-epic-games-integration-platform-epic) for an example supporting both the official client and the alternate Legendary CLI backend.
- See the [EmulationStation Integration (`platform-emulationstation`)](Available-Plugins.md#12-emulationstation-integration-platform-emulationstation) for multi-console ROM directory scanning and title normalization.

### 2. Metadata Plugins
Metadata plugins enrich titles with descriptions, genres, user ratings, and CDN artwork when invoked via **✨ Enhance Metadata**.
- **Keyless Providers**: Standard public APIs (TheGamesDB, ScreenScraper, HowLongToBeat, PCGamingWiki, ProtonDB) function immediately without keys.
- **Authenticated Providers**: Providers requiring API keys (SteamGridDB, IGDB) declare fields in `getSettingsSchema()` and implement `testConnection()`.

### 3. Utility Plugins
Utility plugins run in the background to provide live notification feeds and disk analytics without blocking the user interface.
- See [GamerPower Feeds & Alerts](Available-Plugins.md#1-gamerpower-universal-feeds--alerts-utility-gamerpower) for giveaway and patch note aggregation.
- See [Storage & Drive Analyzer](Available-Plugins.md#2-storage--drive-analyzer-utility-storage) for disk capacity calculation.

### 4. Media Plugins
Media plugins handle multimedia elements such as video trailer previews in the hero banner and ambient game soundtracks.

---

## Registering Plugins

1. Create your module in the appropriate directory:
   - `src/lib/plugins/platforms/<name>.ts`
   - `src/lib/plugins/metadata/<name>.ts`
   - `src/lib/plugins/utility/<name>.ts`
   - `src/lib/plugins/media/<name>.ts`
2. Import and export your plugin instance in `src/lib/plugins/catalog.ts` under its corresponding category array (`PLATFORM_PLUGINS`, `METADATA_PLUGINS`, `UTILITY_PLUGINS`, or `MEDIA_PLUGINS`).
3. If the plugin provides backend Rust detection hooks, register its manifest in `src-tauri/src/plugins/defaults.rs` under `default_plugin_manifests()`.
4. Validate quality gates:
   ```powershell
   npm run check
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

---

## ➡️ Next Steps

- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of all 27 plugins.
- [Technical Wiki: Plugin Developer Guide]({{ '/wiki/Plugin-Developer-Guide/' | relative_url }}) — in-depth technical walkthrough with complete TypeScript templates.
- [Technical Wiki: Plugins Architecture]({{ '/wiki/Plugins-Architecture/' | relative_url }}) — architecture deep-dive.
