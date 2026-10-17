---
layout: docs
title: Available Plugins Catalog
summary: Exhaustive reference of all 27 built-in platform, metadata, utility, and media plugins with features and settings
parent: Documentation Home
nav_order: 5
---

# Available Plugins Catalog

Universal Game Launcher features a modular, in-app plugin framework (`src/lib/plugins/`). All plugins operate as first-class native TypeScript modules compiled directly into the application, guaranteeing zero network CDN vulnerabilities, 100% offline capability, instant loading, and clean decoupling.

Plugins are categorized into four dedicated subdirectories:
- **Platform Plugins (`platforms/`)** — 12 storefront integrations & local launcher engines
- **Metadata Plugins (`metadata/`)** — 11 rich metadata, box art, and compatibility providers
- **Utility Plugins (`utility/`)** — 2 background alert feeds and storage analytics tools
- **Media Plugins (`media/`)** — 2 video trailer and ambient audio enhancement engines

---

## 🕹️ Platform Plugins (`platforms/`)

Platform plugins manage game discovery, launch protocol delegation, social rosters, achievement tracking, and native metadata fetching for specific storefronts and launcher ecosystems.

### 1. Steam Platform Integration (`platform-steam`)
- **Plugin ID**: `platform-steam`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Steam libraries, `appmanifest_*.acf` files, and custom library folders across all mounted drives.
- **Declared Features**: `discovery`, `launch`, `library_vdf`, `cloud_saves`, `store_cdn`, `social`, `achievements`
- **Capabilities**:
  - Direct local VDF manifest discovery across all registered Steam library drives (`libraryfolders.vdf`).
  - Native client launch protocol: `steam://rungameid/{appId}`.
  - Native install & uninstall URIs: `steam://install/{appId}`, `steam://uninstall/{appId}`.
  - Native friends list status protocol: `steam://friends/status`.
  - Native achievements tracker hook: `steam://achievements/{appId}`.
  - Native storefront metadata and CDN artwork query (`queryMetadata`, `queryArtwork`) via official Steam storefront endpoints.
- **Settings Configuration**:
  - `customSteamPath` (Text): Custom path to `steam.exe` if installed in a non-standard location.

---

### 2. Epic Games Integration (`platform-epic`)
- **Plugin ID**: `platform-epic`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Epic Games `.item` manifests with integrated open-source Legendary CLI engine support.
- **Declared Features**: `discovery`, `launch`, `legendary_cli`, `offline_auth`, `eos_overlay`, `social`, `achievements`
- **Capabilities**:
  - Local manifest parsing from `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item`.
  - Default launch protocol: `com.epicgames.launcher://apps/{appId}?action=launch&silent=true`.
  - Alternate launch method: Open-source Legendary CLI headless execution (`legendary launch <appName>`).
  - Native install & uninstall URIs: `com.epicgames.launcher://apps/{appId}?action=install`.
  - Native storefront catalog metadata and keyart fetching (`queryMetadata`, `queryArtwork`).
- **Settings Configuration**:
  - `useLegendary` (Boolean, default: `false`): Enable open-source Legendary CLI as alternate provider backend.
  - `customLegendaryPath` (Text): Full path to installed `legendary.exe` binary.
  - `enableEosOverlay` (Boolean, default: `true`): Inject Epic Online Services overlay.

---

### 3. GOG Galaxy Integration (`platform-gog`)
- **Plugin ID**: `platform-gog`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from GOG Galaxy registry entries, local index databases, and DRM-free installations.
- **Declared Features**: `discovery`, `launch`, `drm_free`, `galaxy_client`, `social`, `achievements`
- **Capabilities**:
  - Queries local Galaxy SQLite database (`galaxy-2.0.db`) and Windows Registry for authentic install locations.
  - Launch protocol: `goggalaxy://openGameView/{gameId}`.
  - Friends list protocol: `goggalaxy://openFriends`.
  - Native metadata and CDN crop artwork queries (`queryMetadata`, `queryArtwork`).
- **Settings Configuration**:
  - `preferDirectExe` (Boolean, default: `false`): Launch standalone GOG executable directly without opening the Galaxy client.
  - `customGalaxyPath` (Text): Custom path to `GalaxyClient.exe`.

---

### 4. itch.io Platform Integration (`platform-itch`)
- **Plugin ID**: `platform-itch`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games installed via the itch.io desktop client and standalone receipt manifests.
- **Declared Features**: `discovery`, `launch`, `indie_installations`, `butler_cli`
- **Capabilities**:
  - Discovers installed games in `%APPDATA%\itch\apps` and receipt files.
  - Launch protocol: `itch://games/{gameId}` or direct executable launch.
  - Query metadata and cover art directly from itch catalog endpoints (`queryMetadata`, `queryArtwork`).
- **Settings Configuration**:
  - `customItchAppsDir` (Text): Custom path to itch games installation directory.

---

### 5. Xbox & Microsoft Store Integration (`platform-xbox`)
- **Plugin ID**: `platform-xbox`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Xbox Game Pass, Microsoft Store packages, and WindowsApps folders.
- **Declared Features**: `discovery`, `launch`, `game_pass`, `uwp_packages`, `social`, `achievements`
- **Capabilities**:
  - Inspects `XboxGames` library volumes, `MicrosoftGame.config`, and package manifests.
  - Launch protocol: `shell:AppsFolder\{PackageFamilyName}!App` or `ms-gamepass://launch?titleId={id}`.
  - Native Xbox app social and achievements integration.
- **Settings Configuration**:
  - `scanMountedDrives` (Boolean, default: `true`): Automatically scan all connected drive letters for `XboxGames` root folders.

---

### 6. EA Desktop Integration (`platform-ea`)
- **Plugin ID**: `platform-ea`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Electronic Arts (EA Desktop & Origin) local installations.
- **Declared Features**: `discovery`, `launch`, `origin_protocol`, `social`, `achievements`
- **Capabilities**:
  - Inspects local EA installer registry entries and configured game folders.
  - Launch protocol: `origin2://game/launch?offerIds={appId}`.
  - Query metadata and category artwork via EA catalog titles (`queryMetadata`, `queryArtwork`).
- **Settings Configuration**:
  - `customEaAppPath` (Text): Custom path to `EADesktop.exe`.

---

### 7. Ubisoft Connect Integration (`platform-ubisoft`)
- **Plugin ID**: `platform-ubisoft`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Ubisoft Connect registry entries and configurations.
- **Declared Features**: `discovery`, `launch`, `uplay_protocol`, `social`, `achievements`
- **Capabilities**:
  - Inspects Ubisoft registry configurations and installed product directories.
  - Launch protocol: `uplay://launch/{gameId}/0`.
  - Friends list and club achievements integration.
- **Settings Configuration**:
  - `customUbisoftPath` (Text): Custom path to `UbisoftConnect.exe`.

---

### 8. Battle.net Integration (`platform-battlenet`)
- **Plugin ID**: `platform-battlenet`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Blizzard Battle.net installations and `battle.net://` URI protocol.
- **Declared Features**: `discovery`, `launch`, `battlenet_protocol`, `social`, `achievements`
- **Capabilities**:
  - Inspects Blizzard installation product databases and system registry.
  - Launch protocol: `battlenet://{appId}` (e.g. `Pro`, `WoW`, `D4`).
- **Settings Configuration**:
  - `customBattleNetPath` (Text): Custom path to `Battle.net.exe`.

---

### 9. Amazon Games Integration (`platform-amazon`)
- **Plugin ID**: `platform-amazon`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Amazon Games client installations and direct game binaries.
- **Declared Features**: `discovery`, `launch`, `amazon_games_protocol`, `nile_cli`
- **Capabilities**:
  - Reads Amazon Games library SQLite manifests and install folders.
  - Launch protocol: `amazon-games://play/{appId}`.
- **Settings Configuration**:
  - `customAmazonGamesDir` (Text): Custom library folder for Amazon Games.

---

### 10. Rockstar Games Integration (`platform-rockstar`)
- **Plugin ID**: `platform-rockstar`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches games from Rockstar Games Launcher installations and `rockstar://` URI scheme.
- **Declared Features**: `discovery`, `launch`, `rockstar_protocol`
- **Capabilities**:
  - Inspects Rockstar Games Launcher manifests and installation folders.
  - Launch protocol: `rockstar://launch/{appId}`.
- **Settings Configuration**:
  - `customLauncherPath` (Text): Custom path to `Launcher.exe`.

---

### 11. Standalone & Local Executables (`platform-standalone`)
- **Plugin ID**: `platform-standalone`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Enables importing and launching arbitrary local games, DRM-free titles, emulators, and custom executables.
- **Declared Features**: `discovery`, `launch`, `custom_executables`, `emulators`, `drm_free`, `manual_import`
- **Capabilities**:
  - Powers the **+ Add Game** modal in the Library sidebar.
  - Direct execution with custom working directory and arguments.
  - Native PE header title derivation and executable inspection (`queryMetadata`, `queryArtwork`).
- **Settings Configuration**:
  - `defaultWorkingDir` (Text): Default directory for standalone game launches.

---

### 12. EmulationStation Integration (`platform-emulationstation`)
- **Plugin ID**: `platform-emulationstation`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers and launches emulated games and ROMs organized via the ES-DE directory standard with automated folder scaffolding and intelligent title normalization.
- **Declared Features**: `rom_scanning`, `folder_scaffolding`, `title_normalization`, `multi_console`
- **Capabilities**:
  - Automatically scans ROM files across all console subdirectories (`nes`, `snes`, `n64`, `gc`, `wii`, `psx`, `ps2`, etc.).
  - Backend directory scaffolding command (`scaffold_emulation_folders`) creating standard folder structures with `systeminfo.txt`.
  - Intelligent ROM title normalization engine stripping dump tags `(USA)`, `[!]`, `(Rev 1)`, GoodTools codes, and NKit extensions.
  - Direct emulation launcher delegation via EmulationStation or configured emulator binaries.
- **Settings Configuration**:
  - `romsPath` (Text, default: `~/ROMs`): Root directory containing console ROM subfolders.
  - `esExecutablePath` (Text): Optional path to EmulationStation / ES-DE executable.
  - `autoScaffold` (Boolean, default: `true`): Automatically scaffold missing console directories.

---

## 📚 Metadata Plugins (`metadata/`)

Metadata plugins enrich titles with descriptions, genres, release dates, user ratings, and CDN artwork. They are run manually on-demand by the user via the **✨ Enhance Metadata** action in the showcase view and the Manage Game modal to fill missing fields.

### 1. Steam Store & Web API Provider (`metadata-steam`)
- **Plugin ID**: `metadata-steam`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Official Steam Storefront metadata, CDN artwork, reviews, achievements, and Steam Web API synchronization.
- **Declared Features**: `store_metadata`, `cdn_artwork`, `achievements`, `owned_library`, `web_api`
- **Requires API Key**: Optional (`requiresApiKey: false`)
- **Capabilities**:
  - Keyless store details query via `store.steampowered.com/api/appdetails`.
  - High-resolution Cloudflare CDN covers and 1080p screenshots.
  - Optional Steam Web API integration for owned games list and playtime sync.
- **Settings Configuration**:
  - `steamWebApiKey` (Password): Optional Steam Web API Key from `steamcommunity.com/dev/apikey`.
  - `steamId64` (Text): 17-digit SteamID64 for player library synchronization.

---

### 2. SteamGridDB Community Artwork Provider (`metadata-steamgriddb`)
- **Plugin ID**: `metadata-steamgriddb`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Provides access to community-crafted custom capsule covers, hero banners, and logos via the SteamGridDB API.
- **Declared Features**: `custom_artwork`, `community_grids`, `heroes`, `logos`, `icons`
- **Requires API Key**: Yes (`requiresApiKey: true`)
- **Capabilities**:
  - Queries `www.steamgriddb.com/api/v2` for 600×900 vertical capsules, 1920×620 heroes, transparent logos, and square icons.
  - Direct in-memory connection testing (`testConnection`).
- **Settings Configuration**:
  - `apiKey` (Password, Required): API key from `steamgriddb.com/profile/preferences/api`.
  - `allowNsfw` (Boolean, default: `false`): Allow NSFW-tagged community artwork.
  - `allowHumor` (Boolean, default: `true`): Include humorous community parody artwork.

---

### 3. IGDB Comprehensive Gaming Metadata Provider (`metadata-igdb`)
- **Plugin ID**: `metadata-igdb`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Queries the Internet Game Database (IGDB) by Twitch for global taxonomy, developer lists, themes, and critic ratings.
- **Declared Features**: `game_taxonomy`, `critic_reviews`, `developer_credits`, `franchises`
- **Requires API Key**: Yes (`requiresApiKey: true`)
- **Capabilities**:
  - Queries `api.igdb.com/v4/games` using Twitch OAuth Client Credentials.
  - Complete game taxonomy, franchise links, developer studios, and aggregated critic ratings.
  - Direct connection verification.
- **Settings Configuration**:
  - `clientId` (Text, Required): Twitch Application Client ID from `dev.twitch.tv/console`.
  - `clientSecret` (Password, Required): Twitch Application Client Secret.

---

### 4. TheGamesDB Retro Metadata Provider (`metadata-thegamesdb`)
- **Plugin ID**: `metadata-thegamesdb`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Comprehensive community video game database specializing in retro console ROMs, box art, fanart heroes, clear logos, and release taxonomy.
- **Declared Features**: `retro_metadata`, `boxart`, `fanart`, `clearlogos`, `taxonomy`, `screenshots`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `api.thegamesdb.net/v1/` for authentic retro console releases across NES, SNES, N64, Genesis, PSX, and handhelds.
  - 2D/3D box art, fanart wallpaper banners, clear logos, and developer credits.
- **Settings Configuration**:
  - `apiKey` (Password): Optional developer API key for higher rate limits.

---

### 5. ScreenScraper Multi-Console ROM Provider (`metadata-screenscraper`)
- **Plugin ID**: `metadata-screenscraper`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Keyless retro gaming metadata and artwork provider supporting 2D/3D box art, wheel logos, screenshot titles, and video snaps from screenscraper.fr.
- **Declared Features**: `retro_metadata`, `boxart_2d_3d`, `wheel_logos`, `video_snaps`, `screenshots`, `multi_language`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries screenscraper.fr endpoints matching ROM file hashes or normalized titles.
  - 2D and 3D authentic box art, arcade wheel logos, screenshot titles, and video clips.
- **Settings Configuration**:
  - `devId` (Text): Optional developer identifier.
  - `devPassword` (Password): Optional developer password.
  - `preferredRegion` (Select: `us`, `eu`, `jp`, `wor`): Preferred regional box art.

---

### 6. GOG Catalog Metadata & CDN Artwork (`metadata-gog`)
- **Plugin ID**: `metadata-gog`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Official GOG catalog endpoints for store artwork, game summaries, DRM-free releases, and system requirements.
- **Declared Features**: `store_artwork`, `drm_free_meta`, `system_requirements`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `embed.gog.com` for official vertical covers, crop wallpapers, and DRM-free descriptions.
- **Settings Configuration**:
  - `language` (Select, default: `en`): Preferred catalog language.

---

### 7. Epic Games Store Catalog & Artwork (`metadata-epic`)
- **Plugin ID**: `metadata-epic`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Fetches Epic Games Store catalog manifests, promotional artwork covers, and release dates.
- **Declared Features**: `promotional_artwork`, `catalog_manifests`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `store-content.ak.epicgames.com` for official catalog manifests, wide keyart, and publisher descriptions.
- **Settings Configuration**:
  - `country` (Text, default: `US`): Two-letter country code for regional store catalog.

---

### 8. itch.io Indie Games Provider (`metadata-itch`)
- **Plugin ID**: `metadata-itch`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Discovers indie game titles, developer profiles, screenshots, and account library synchronization from itch.io.
- **Declared Features**: `indie_meta`, `screenshots`, `developer_credits`, `library_sync`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `itch.io/api/1` for indie creator profiles, screenshots, and game descriptions.
- **Settings Configuration**:
  - `apiKey` (Password): Optional itch.io API key for syncing owned purchases.

---

### 9. HowLongToBeat Completion Times (`metadata-howlongtobeat`)
- **Plugin ID**: `metadata-howlongtobeat`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Crowd-sourced gameplay completion metrics (Main Story, Main + Extra, Completionist) from HowLongToBeat.
- **Declared Features**: `gameplay_length`, `completionist_stats`, `speedrun_times`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries HowLongToBeat search API for average gameplay hours.
  - Adds playtime metrics directly to the game showcase overview.
- **Settings Configuration**:
  - None required; works immediately keyless.

---

### 10. PCGamingWiki Fixes & Compatibility (`metadata-pcgamingwiki`)
- **Plugin ID**: `metadata-pcgamingwiki`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Supplies controller support flags, ultra-wide FOV fixes, cloud save locations, and community fixes from PCGamingWiki.
- **Declared Features**: `compatibility`, `fov_fixes`, `controller_support`, `save_locations`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `www.pcgamingwiki.com/w/api.php` for game engine, save paths, configuration directories, and widescreen fixes.
- **Settings Configuration**:
  - None required; works immediately keyless.

---

### 11. ProtonDB Linux & Steam Deck Tiers (`metadata-protondb`)
- **Plugin ID**: `metadata-protondb`
- **Version**: `1.0.0` · **Author**: `HELIX Origin`
- **Description**: Community compatibility ratings (Native, Platinum, Gold, Silver, Bronze, Borked) and recommended launch commands.
- **Declared Features**: `deck_verified`, `proton_tier`, `launch_tweaks`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Queries `www.protondb.com/api/v1` for Proton tier ratings and Steam Deck verified status.
- **Settings Configuration**:
  - None required; works immediately keyless.

---

## 🛠️ Utility Plugins (`utility/`)

Utility plugins provide background services, live notifications, free game giveaways, store specials, and drive capacity calculations.

### 1. GamerPower Universal Feeds & Alerts (`utility-gamerpower`)
- **Plugin ID**: `utility-gamerpower`
- **Version**: `1.0.0` · **Author**: `HELIX-Origin`
- **Description**: Universal aggregator for 100% free game giveaways, verified store promotions & deals, and game patch notes across Steam, Epic Games, GOG, itch.io, Xbox, and more via the official GamerPower API.
- **Declared Features**: `notifications`, `giveaways`, `free_games`, `store_deals`, `promotions`, `patch_notes`, `game_updates`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Live 100% free giveaway alerts on the Dashboard.
  - Store promotion and discount bargains with minimum percentage threshold filtering.
  - Developer update notes and patch logs for installed library titles.
  - Desktop push notifications for newly announced giveaways.
- **Settings Configuration**:
  - `enableFreeGames` (Boolean, default: `true`): Monitor and display 100% free game giveaways.
  - `enableDeals` (Boolean, default: `true`): Display store promotions and sales.
  - `enablePatchNotes` (Boolean, default: `true`): Fetch patch notes for installed games.
  - `minDiscountPercent` (Number, default: `50`): Minimum discount percentage required for deal alerts.
  - `enableNotifications` (Boolean, default: `true`): Show system notifications for newly announced deals.

---

### 2. Storage & Drive Analyzer (`utility-storage`)
- **Plugin ID**: `utility-storage`
- **Version**: `1.0.0` · **Author**: `HELIX-Origin`
- **Description**: Calculates disk footprint across all platform directories and library drives using non-blocking background workers.
- **Declared Features**: `disk_analysis`, `drive_reporting`, `non_blocking_worker`
- **Requires API Key**: No (`requiresApiKey: false`)
- **Capabilities**:
  - Powers the `/storage` view with drive capacity visualization and per-game disk space analysis.
  - Offloaded to `tauri::async_runtime::spawn_blocking` for 0ms UI blocking.
- **Settings Configuration**:
  - `includeTemporaryFiles` (Boolean, default: `false`): Include temporary shader caches in storage calculation.

---

## 🎞️ Media Plugins (`media/`)

Media plugins enhance the game showcase with dynamic multimedia playback.

### 1. Video Trailer Player (`media-trailers`)
- **Plugin ID**: `media-trailers`
- **Version**: `1.0.0` · **Author**: `HELIX-Origin`
- **Description**: Streams and previews official video trailers in the game detail hero banner.
- **Declared Features**: `video_playback`, `trailers`
- **Capabilities**:
  - Plays official webm/mp4 video trailers directly inside the showcase hero section.
- **Settings Configuration**:
  - `autoPlay` (Boolean, default: `false`): Automatically play trailers on selection.
  - `muteAudio` (Boolean, default: `true`): Mute audio during trailer playback.

---

### 2. Game Audio & Soundtracks (`media-ambient`)
- **Plugin ID**: `media-ambient`
- **Version**: `1.0.0` · **Author**: `HELIX-Origin`
- **Description**: Plays ambient background themes and official game soundtracks on selection.
- **Declared Features**: `audio_playback`, `ambiance`
- **Capabilities**:
  - Streams ambient background music when browsing game details.
- **Settings Configuration**:
  - `volume` (Number, default: `0.3`): Ambient playback volume (0.0 to 1.0).

---

## ➡️ Next Steps

- [Plugin Development Guide](Plugin-Development.md) — How to build and contribute new plugins
- [Metadata Providers Guide](Metadata-Providers.md) — Role hierarchy and metadata enhancement
- [Settings & Plugin Configuration](Settings-and-API-Keys.md) — Configuring credentials
