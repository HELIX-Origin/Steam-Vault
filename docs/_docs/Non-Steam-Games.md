---
layout: docs
title: Manually Added Local Games
summary: Adding, organizing, and enriching standalone local games, custom executables, and ROMs
parent: Documentation Home
nav_order: 7
---

# Manually Added Local Games

Universal Game Launcher natively discovers and manages games across all major game launchers — including **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA Desktop**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**.

In addition to automatic launcher detection, Universal Game Launcher provides complete support for **manually adding local games, standalone DRM-free executables, indie releases, and custom shortcuts** so your entire collection lives in one unified library.

---

## 📁 Adding Local Games (`+ Add Game`)

To add a local game or standalone executable:
1. Open the **Library** view (`/library`).
2. Click the **+ Add Game** button at the top of the left games list sidebar.
3. In the modal:
   - **Browse Executable:** Click **Browse...** to select your `.exe`, `.bat`, `.cmd`, or script binary.
   - **Automatic Title Derivation:** The game title is automatically generated from the executable filename or parent directory using the **Universal Title Normalizer**.
   - **Platform Attribution:** Categorize the game under **Standalone**, or attribute it to a specific storefront or publisher.
   - **Launch Arguments & Working Directory:** Specify custom command-line parameters and working folders.
   - **Custom Cover Art:** Optionally browse and select a local cover image (`.png`, `.jpg`).
4. Click **Save Game**. The game appears immediately in your library under its platform accordion.

---

## 🔤 Intelligent Title Normalization (`utility-title-normalizer`)

Standalone and indie games often arrive with awkward directory names, compressed slugs, or release tags. The built-in **Universal Title Normalizer** automatically cleans these titles:

- **RPG Maker Engine Parsing**: Automatically checks child folders for `www/data/System.json` or `data/System.json` and reads the authentic `"gameTitle"` property (e.g. resolving `relicts-auf-aeson` to **`Relicts of Aeson`**).
- **Slug De-concatenation**: Decomposes concatenated lowercase folder names into natural English titles (e.g. `myfutafamily` to **`My Futa Family`**).
- **Article Inversion**: Converts inverted library titles to natural reading order (e.g. `Legend of Zelda, The - The Wind Waker` to **`The Legend of Zelda: The Wind Waker`**).
- **Deduplication**: Automatically groups identical executables and prevents duplicate entries in `library:custom_games`.

---

## ⚙️ Per-Game Customization Modal (`⚙ Manage`)

Once added, any standalone title can be fully customized by clicking **⚙ Manage** in the showcase header:

- **Title (Display Name)**: Edit the display name at any time.
- **Short Description**: Provide a custom synopsis or plot overview.
- **Developers & Publishers**: Authoritative creator and studio credits.
- **Release Date**: Formatted release date.
- **Genres**: Custom category tag pills.
- **Custom Artwork Slots**:
  - **Portrait Capsule Cover (600×900)**: Direct image URL or local file picker (`Browse...`).
  - **Hero Wallpaper Banner (16:9)**: Direct image URL or local file picker (`Browse...`).
  - **Transparent Logo**: Direct image URL or local file picker (`Browse...`).

---

## 🤖 Automatic Multi-Source Metadata Resolution

Standalone games benefit from the **Universal Metadata Engine**:
- The engine queries **Steam Store Search**, **GOG Catalog v1**, and **Wikipedia / Wikidata MediaWiki API** to automatically match game details and synopses.
- If community artwork is desired, configuring a free **SteamGridDB** API key in **Settings → Plugin Settings** enables automatic 600×900 capsules and hero backdrops.

---

## 🛡️ Persistence & Safe Isolation

All metadata, custom tags, artwork slot assignments, and collection memberships for manually added games are persisted safely in the generic embedded SQLite database (`database.sqlite`) and in-memory Redis KV tier (`mem_kv`).
- Universal Game Launcher **never writes to Steam's `shortcuts.vdf`** or external client files.
- Manually added games operate with full independence.

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md) — showcase layout and unified media tabs.
- [Artwork Management](Artwork-Management.md) — custom covers, heroes, logos, and icons.
- [Metadata Providers](Metadata-Providers.md) — multi-source metadata engine.
- [Collections](Collections.md) — grouping titles across platforms.
