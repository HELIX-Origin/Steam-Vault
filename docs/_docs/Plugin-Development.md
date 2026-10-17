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
├── engine.ts              # In-memory registry, KV persistence, metadata resolution
├── catalog.ts             # Master catalog exporting all active Universal Core plugins
├── index.ts               # Public entrypoint and helper re-exports
├── platforms/
│   └── universal.ts       # platform-universal (12 platform specs + global helpers)
├── metadata/
│   └── universal.ts       # metadata-universal (automatic multi-source cascade)
├── utility/
│   ├── title-normalizer.ts# utility-title-normalizer (clean titles, RPG Maker & ROM normalizer)
│   ├── deals.ts           # utility-deals (100% free giveaways & store deals)
│   ├── patchnotes.ts      # utility-patchnotes (developer game update feeds)
│   └── storage.ts         # utility-storage (non-blocking drive footprint analyzer)
└── media/
    └── universal.ts       # media-universal (Steam CDN MP4 trailer streaming & soundtracks)
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

## Universal Core Plugins & Global Helpers Pattern

In Universal Game Launcher v1.4.0, plugins are designed as **Universal Core Plugins** paired with **Global Helper Modules**:

### 1. Platform Universal Engine (`platform-universal`)
Consolidates all 12 platform specifications into a unified engine and exports global helpers (`getAllPlatformSpecs()`, `detectPlatformLauncher()`, `installPlatformLauncher()`, `getPlatformLaunchSpec()`, `getPlatformUris()`). This keeps platform logic centralized and easily reusable across views.

### 2. Metadata Universal Engine (`metadata-universal`)
Cascades automatically across Local Inspector, Libretro CDN, Steam Store, GOG, Wikipedia/Wikidata, PCGamingWiki, ProtonDB, and optional authenticated APIs.

### 3. Dedicated Title Normalizer (`utility-title-normalizer`)
Operates cleanly in both frontend TypeScript and backend Rust (`src-tauri/src/plugins/title_normalizer.rs`) to parse RPG Maker `System.json` titles, split lowercase slugs, invert articles, and clean ROM dump tags.

### 4. Media Universal Engine (`media-universal`)
Extracts modern Steam CDN MP4 trailer streams and manages ambient theme audio inside the consolidated media showcase.

---

## Registering Plugins

1. Author your module under `src/lib/plugins/`:
   - `src/lib/plugins/platforms/universal.ts`
   - `src/lib/plugins/metadata/universal.ts`
   - `src/lib/plugins/utility/<feature>.ts`
   - `src/lib/plugins/media/universal.ts`
2. Export your plugin instance and register it in `src/lib/plugins/catalog.ts`.
3. If the plugin defines backend Rust detection hooks, register its manifest in `src-tauri/src/plugins/defaults.rs` under `default_plugin_manifests()`.
4. Validate quality gates:
   ```powershell
   npm run check
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

---

## ➡️ Next Steps

- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of Universal Core plugins.
- [Technical Wiki: Plugin Developer Guide]({{ '/wiki/Plugin-Developer-Guide/' | relative_url }}) — in-depth technical walkthrough with complete TypeScript templates.
- [Technical Wiki: Plugins Architecture]({{ '/wiki/Plugins-Architecture/' | relative_url }}) — architecture deep-dive.
