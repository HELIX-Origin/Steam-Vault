---
layout: docs
title: Artwork Management
summary: Applying grid, hero, logo, and icon art across all games and manually added local titles
parent: Documentation Home
nav_order: 13
---

# Artwork Management

The **Custom Artwork Studio** in Universal Game Launcher allows you to customize the visual assets displayed for any title in your library — across all supported storefront games, manually added local executables, and emulator ROMs. You can fetch high-resolution artwork from community providers such as **SteamGridDB**, extract official storefront assets, query retro box art from **TheGamesDB** and **ScreenScraper**, or apply your own local image files.

---

## 🖼️ Standardized Artwork Slots

Universal Game Launcher manages five standardized artwork slots for every title:

| Slot | File / Key Identifier | Recommended Dimensions | Format |
| :--- | :--- | :--- | :--- |
| **Portrait Capsule** | `<appId>_capsule` | 600×900 (2:3 aspect ratio) | PNG or JPEG |
| **Hero Banner** | `<appId>_hero` | 1920×620 or 1920×1080 (16:9) | PNG or JPEG |
| **Header Grid** | `<appId>_header` | 920×430 | PNG or JPEG |
| **Logo** | `<appId>_logo` | Variable width / height | Transparent PNG |
| **Icon** | `<appId>_icon` | 256×256 (1:1 square) | PNG or ICO |

Universal Game Launcher renders images using hardware-accelerated transforms, ensuring crisp visuals across all screen resolutions without UI blur overhead.

---

## 📂 Local Artwork Storage & Isolation

Universal Game Launcher operates completely independently from external client folders:

- **Local Persistence**: Custom artwork and downloaded assets are stored in the application's local data directory under:
  ```plaintext
  %APPDATA%\universal-game-launcher\artwork\
  ```
- **Fast KV Indexing**: File mappings are recorded directly in `database.sqlite` and mirrored in the in-memory Redis KV tier (`artwork_cache`) for 0ms retrieval on startup.
- **Zero External Client File Writes**: Universal Game Launcher **never** writes to or alters Steam's `userdata/<account-id>/config/grid/` directory or other launcher caches. Your library customizations remain pristine and self-contained.

---

## 🌐 Sourcing Artwork Across Providers

1. Open the game in the **Library Showcase View** (`/library`) and scroll to the **Custom Artwork Studio** section (or click **Manage → Artwork**).
2. Choose the artwork slot you want to configure: **Portrait Capsule**, **Hero Banner**, **Logo**, or **Icon**.
3. Use the search input or provider tabs to query assets:
   - **SteamGridDB**: Community-crafted vertical capsules, hero wallpapers, transparent logos, and square icons. (Requires free API key in **Settings → Plugin Settings**).
   - **Steam Store**: Official store capsules, wide banners, and community backgrounds.
   - **GOG Catalog**: Official vertical covers and wide galaxy banners.
   - **TheGamesDB**: Authentic retro box art, clear logos, and fanart banners for console ROMs.
   - **ScreenScraper**: 2D/3D retro box art, arcade wheel logos, and screenshot titles.
   - **IGDB**: High-resolution box art from Twitch's video game database.
   - **PCGamingWiki**: Original physical box covers and community artwork.
   - **Local File Import**: Select any local PNG, JPG, or JPEG file from your computer.
4. Click any artwork card to apply it immediately. The showcase view and sidebar update reactively with zero reload lag.

---

## 🛡️ Cache Invariant & Provider Overwrites

When you assign custom artwork or enhance artwork via metadata providers:
- The artwork mapping is preserved locally in `database.sqlite`.
- Background platform scans and provider syncs **will never overwrite** your selected custom artwork.
- If you wish to revert to official storefront defaults, click **↺ Reset Cache to Provider Defaults** in the game's Manage modal.

---

## ➡️ Next Steps

- [Metadata Providers](Metadata-Providers.md) — role hierarchy and metadata enhancement.
- [Available Plugins Catalog](Available-Plugins.md) — complete catalog of all 27 plugins.
- [Organizing Your Library](Organizing-Your-Library.md) — customizing library views.
