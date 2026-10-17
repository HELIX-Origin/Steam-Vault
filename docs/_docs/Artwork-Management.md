---
layout: docs
title: Artwork Management
summary: Applying grid, hero, logo, and icon art across all games and manually added local titles
parent: Documentation Home
nav_order: 13
---

# Artwork Management

The **Custom Artwork Studio** in Universal Game Launcher allows you to customize the visual assets displayed for any title in your library — across all supported storefront games, manually added local executables, and emulator ROMs. You can fetch high-resolution artwork from community providers such as **SteamGridDB**, extract official storefront assets, or apply your own local image files.

---

## 🖼️ The Visual Slots

Universal Game Launcher manages five standardized artwork slots for every title:

| Slot | File / Key Identifier | Recommended Dimensions | Format |
| :--- | :--- | :--- | :--- |
| **Portrait Grid** | `<appid>p` / `capsule` | 600×900 (2:3 aspect ratio) | PNG or JPEG |
| **Landscape Grid** | `<appid>` / `grid` | 920×430 | PNG or JPEG |
| **Hero Banner** | `<appid>_hero` / `hero` | 1920×620 or 1920×1080 (16:9) | PNG or JPEG |
| **Logo** | `<appid>_logo` / `logo` | Variable width / height | Transparent PNG |
| **Icon** | `<appid>_icon` / `icon` | 256×256 (1:1 square) | PNG or ICO |

Universal Game Launcher writes and references images with extensions matching their native formats (`.png`, `.jpg`, `.jpeg`), ensuring crisp hardware-accelerated rendering.

---

## 📂 Where Artwork is Stored

- **Storefront & Local Titles**: Stored in the local application data directory under `%APPDATA%\universal-game-launcher\artwork\` and indexed directly in `database.sqlite` via the in-memory KV tier.
- **Steam User Data Synchronization**: For Steam titles, Universal Game Launcher also synchronizes with Steam's native custom grid directory:
  ```plaintext
  userdata/<account-id>/config/grid/
  ```
  This ensures that custom artwork configured inside Universal Game Launcher also renders natively if you launch the Steam client directly.

---

## 🌐 Applying Artwork from Multiple Providers

1. Open the game in the **Library Showcase View** (`/library`) and scroll to the **Custom Artwork Studio** section (or click **Manage → Artwork**).
2. Choose the artwork slot you want to configure: **Portrait Capsule**, **Hero Banner**, **Logo**, or **Icon**.
3. Use the search input to query titles across providers:
   - **SteamGridDB**: Community portrait grids, hero wallpapers, transparent logos, and square icons. (Configurable via **Settings → Plugin Settings**).
   - **Steam Store**: Official store capsules, wide headers, and community wallpapers.
   - **GOG Catalog**: High-resolution vertical box art and galaxy banners.
   - **PCGamingWiki**: Authentic original physical box art and wiki infobox covers.
   - **Wikipedia**: Encyclopedic cover scans and historical box artwork.
   - **VNDB**: Visual novel covers and Japanese character banners.
   - **IGDB**: High-resolution box art from Twitch's video game database.
4. Filter by static or animated art, or toggle NSFW content filtering.
5. Click any card to apply the artwork instantly. Dead or unreachable links are filtered out automatically.

---

## ⚡ Automatic Artwork Fetching

Universal Game Launcher automatically avoids blank or missing posters:

- **Automatic on Import**: When discovering games from storefront launchers or importing manually added local titles, the launcher automatically downloads missing artwork using SteamGridDB and Steam Store Search.
- **One-Click Library Refresh**: In **Settings**, click **Library Metadata Refresh** to perform a background pass and fetch missing artwork for all library titles without interrupting playback.

---

## 📁 Applying Artwork from Local Files

1. Select the title and the target slot, then select **From File…**.
2. Select or enter the path to a PNG or JPEG file on your drive and click **Apply**.
3. Images up to 64 MiB are supported. Transparent PNGs are recommended for the **Logo** slot to avoid solid background boxes.

---

## 🗑️ Removing Artwork

If a slot contains custom artwork, clicking **Remove** clears the custom override and reverts the game to its default storefront asset. Previous custom images are copied to a timestamped backup directory (`universal-game-launcher/backups/<timestamp>/`), ensuring no assets are lost.

---

## 💡 Practical Tips

- **Transparent Logos**: Always use transparent PNGs for the **Logo** slot so the artwork floats cleanly over the 16:9 hero wallpaper.
- **Hero Wallpaper Vignette**: Universal Game Launcher automatically applies an obsidian gradient vignette across hero banners, ensuring title text and action buttons remain readable.
- **Offline Reliability**: Once fetched or applied, all artwork is cached locally on disk for instantaneous offline access.

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md) — viewing games in the showcase view.
- [Manually Added Local Games](Non-Steam-Games.md) — customizing standalone executables.
- [Plugin Development](Plugin-Development.md) — building custom artwork and metadata plugins.
