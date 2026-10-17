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
   - **Automatic Title Derivation:** The game title is automatically generated from the executable filename with clean capitalization, which you can edit directly.
   - **Platform Attribution:** Categorize the game under **Standalone**, or attribute it to a specific storefront or publisher.
   - **Launch Arguments & Working Directory:** Specify custom command-line parameters and working folders.
   - **Custom Cover Art:** Optionally browse and select a local cover image (`.png`, `.jpg`).
4. Click **Save Game**. The game appears immediately in your library under its platform accordion.

---

## 🖼️ Artwork with SteamGridDB & Local Files

Standalone and manually added games don't come with storefront CDN artwork. Universal Game Launcher automatically matches titles using **SteamGridDB** and **Steam Store Search** to find:
- **Portrait Capsules** (600×900 vertical capsule covers)
- **Hero Banners** (1920×620 / 1920×1080 wide backdrop art)
- **Logos** (transparent title treatment)
- **Icons** (high-resolution square icons)

You can also drop your own PNG or JPEG images directly into the dedicated [Artwork Management](Artwork-Management.md) view for any title.

> **Note:** SteamGridDB requires an optional API key. Configure one under **Settings → Plugin Settings** to enable automated high-resolution artwork discovery.

---

## ✨ Enhancing Standalone Game Metadata

After adding a local game, you can enrich its description, genres, developer credits, and release year using third-party metadata providers:
1. Select the title in the Library showcase view.
2. Click **✨ Enhance Metadata** (or open **Manage → Manage Metadata**).
3. Select **TheGamesDB**, **ScreenScraper**, **IGDB**, **PCGamingWiki**, or **HowLongToBeat**.
4. The provider queries the title and fills missing metadata fields into your local database.

---

## 🛡️ Persistence & Safe Isolation

All metadata, custom tags, artwork slot assignments, and collection memberships for manually added games are persisted safely in the generic embedded SQLite database (`database.sqlite`) and in-memory Redis KV tier (`mem_kv`).
- Universal Game Launcher **never writes to Steam's `shortcuts.vdf`** or external client files.
- Manually added games operate with full independence.

---

## ➡️ Next Steps

- [Supported Launchers & Platforms](External-Platforms.md) — multi-launcher discovery.
- [Artwork Management](Artwork-Management.md) — custom covers, heroes, logos, and icons.
- [Metadata Providers](Metadata-Providers.md) — role hierarchy and metadata enhancement.
- [Collections](Collections.md) — grouping titles across platforms.
