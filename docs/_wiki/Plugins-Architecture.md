---
layout: wiki
title: Plugins Architecture & Ecosystem
summary: Deep dive into the Universal Core plugins framework, global helper modules, and automatic multi-source metadata resolution
parent: Technical Wiki
nav_order: 9
---

# Plugins Architecture & Ecosystem

Universal Game Launcher implements an in-app native **Universal Core Plugins framework** (`src/lib/plugins/`) designed for instantaneous game discovery, automatic multi-source metadata enrichment, title normalization, and multimedia capability integration with zero external network failure points or loose JSON manifest files.

---

## Architectural Principles

1. **Universal Core Architecture**: Rather than fragmenting logic across dozens of isolated, duplicate files, platform and metadata capabilities are consolidated into unified **Universal Core Plugins** (`platform-universal`, `metadata-universal`, `utility-title-normalizer`, `media-universal`, `utility-deals`, `utility-patchnotes`, `utility-storage`) paired with **Global Helper Modules**.
2. **Native In-App Modular Framework**: All plugins live directly within the application codebase under `src/lib/plugins/`. This completely eliminates Cloudflare network blockages, CDN rate limits, runtime `.mjs` eval vulnerabilities, and fragile external JSON manifests.
3. **Global Helper Modules**: Universal Core plugins export strongly typed global helper functions (e.g. `getAllPlatformSpecs()`, `detectPlatformLauncher()`, `installPlatformLauncher()`, `getPlatformLaunchSpec()`, `getPlatformUris()`) so views and background workers share unified logic without duplication.
4. **Dedicated Title Normalizer (`utility-title-normalizer`)**: Title normalization is extracted into its own dedicated utility plugin and backend Rust module (`src-tauri/src/plugins/title_normalizer.rs`) to ensure clean titles across RPG Maker, itch.io, standalone executables, and emulator ROMs.
5. **Automatic Multi-Source Metadata Cascade (`metadata-universal`)**: Metadata resolution runs automatically in the background across Local Inspector, Libretro CDN, Steam Store, GOG Catalog v1, Wikipedia/Wikidata MediaWiki API, PCGamingWiki Cargo API, and ProtonDB.
6. **Modern Steam CDN Trailer Streaming (`media-universal`)**: Automatically extracts Akamai CDN MP4 video trailers (`movie_max.mp4`) from Steam Store responses and streams them in the unified media showcase.
7. **Platform Providers with Alternate Launcher Backends**: The `platform-universal` plugin defaults to official storefront client protocols while seamlessly providing an optional toggle to use the open-source **Legendary CLI** engine for Epic Games when configured.
8. **Universal Non-Blocking Background Workers**: Heavy filesystem operations (such as drive space scanning in `utility-storage` and library scans) run asynchronously via `tauri::async_runtime::spawn_blocking` and Svelte background workers. Switching tabs never freezes the UI.
9. **Settings-Only Cache Management**: Maintenance operations (cache refresh and cache purge) live cleanly in **Settings → Storage & Cache**, protecting user customizations and keeping Library views focused on playing and customization.
10. **Air-Gapped Local Operation**: The application functions 100% offline out of the box.

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
│  │   Universal Core Plugins  │  │   SQLite Database │  │
│  │  platforms/universal.ts   │  │  database.sqlite  │  │
│  │  metadata/universal.ts    │  │                   │  │
│  │  utility/title-normalizer │  │                   │  │
│  │  media/universal.ts       │  │                   │  │
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

## ➡️ Next Steps

- [Plugin Developer Guide](Plugin-Developer-Guide.md) — step-by-step developer tutorial with templates.
- [Architecture](Architecture.md) — high-level system overview.
- [Backend (Rust)](Backend-Rust.md) — IPC commands and database layer.
- [Frontend (SvelteKit)](Frontend-SvelteKit.md) — UI architecture and Svelte 5 Runes.
