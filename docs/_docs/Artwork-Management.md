---
layout: docs
title: Artwork Management
summary: Standardized artwork slots, local asset caching, and customizing art via the Manage Game modal
parent: Documentation Home
nav_order: 13
---

# Artwork Management

Universal Game Launcher provides flexible artwork management across all supported storefront games, manually added local executables, and emulator ROMs. You can rely on automatic multi-source artwork resolution (Steam Store CDN, Libretro Thumbnails CDN, GOG), fetch high-resolution assets from community providers such as **SteamGridDB**, or apply your own local image files via the **⚙ Manage** modal.

---

## 🖼️ Standardized Artwork Slots

Universal Game Launcher manages standardized artwork slots for every title:

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

## ⚙️ Customizing Artwork via `⚙ Manage`

To assign custom artwork to any title:

1. Open the game in the **Library Showcase View** (`/library`) and click the **⚙ Manage** button in the showcase action header.
2. In the **Manage Game** modal, navigate to the artwork input sections:
   - **Portrait Capsule Cover (600×900)**: Enter an image URL or click **Browse...** to select a local `.png` or `.jpg` file.
   - **Hero Wallpaper Banner (16:9)**: Enter an image URL or click **Browse...** to pick a wide banner image.
   - **Transparent Logo**: Enter an image URL or click **Browse...** to pick a transparent PNG title logo.
3. Click **Save Changes**. The showcase view and sidebar update reactively with zero reload lag.

---

## 🤖 Automatic Multi-Source Artwork Resolution

For games without manual overrides:
- **PC Games**: The Steam Store API and GOG Catalog v1 automatically populate vertical capsules, wide hero wallpapers, and official screenshots.
- **Console ROMs**: The Libretro Thumbnails CDN automatically resolves authentic 2D/3D box art (`Named_Boxarts`), title screens (`Named_Titles`), and logos based on the normalized ROM title.
- **Optional Community Art**: If a free **SteamGridDB** API key is configured in **Settings → Plugin Settings**, the launcher can query community-crafted capsules, animated heroes, and clear logos.

---

## 🛡️ Cache Invariant & Provider Overwrites

When you assign custom artwork:
- The artwork mapping is preserved locally in `database.sqlite` under `game_meta:<appId>`.
- Background platform scans and provider syncs **will never overwrite** your selected custom artwork.
- To refresh official storefront assets without losing custom overrides, use **↻ Refresh Metadata Cache** in **Settings → Storage & Cache**.
- To purge all cached images and rebuild from scratch, use **🗑 Purge Data Cache** in **Settings → Storage & Cache**.

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md) — showcase layout and unified media tabs.
- [Metadata Providers](Metadata-Providers.md) — automatic multi-source metadata engine.
- [Settings & Plugin Configuration](Settings-and-API-Keys.md) — storage & cache controls.
