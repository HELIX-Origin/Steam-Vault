---
layout: docs
title: Manually Added Local Games
summary: Adding, organizing, and enriching standalone local games and custom executables
parent: Documentation Home
nav_order: 7
---

# Manually Added Local Games

Universal Game Launcher natively discovers and manages games across all major game launchers — including **Steam**, **Epic Games Store**, **GOG Galaxy**, **Ubisoft Connect**, **EA App**, **Xbox / Microsoft Store**, **Battle.net**, **itch.io**, and **Amazon Games**.

In addition to automatic launcher detection, Universal Game Launcher provides complete support for **manually adding local games, standalone DRM-free executables, and indie releases** so your entire collection lives in one unified library.

## 📁 Adding Local Games

When you add a standalone executable or local installation:
- **Executable & Launch Parameters:** Set custom paths, working directories, and command-line arguments.
- **Platform Attribution:** Tag games with specific publisher or storefront origins, or classify them under **Other**.
- **Unified Launching:** Manually added games launch with the same responsive one-click controls as store-installed games.

## 🖼️ Artwork with SteamGridDB & Local Files

Standalone and manually added games don't come with storefront CDN artwork. Universal Game Launcher automatically matches titles using **SteamGridDB** and **Steam Store Search** to find:
- **Portrait Grids** (vertical capsule covers)
- **Hero Banners** (wide background art)
- **Logos** (transparent title treatment)
- **Icons** (high-resolution square icons)

You can also drop your own PNG or JPEG images directly into the dedicated [Artwork Management](Artwork-Management.md) view for any title.

> **Note:** SteamGridDB requires an optional API key. Configure one under **Settings & API Keys** to enable automated high-resolution artwork discovery.

## 🎮 Multi-Store Launcher Integration

Rather than adding every game by hand, Universal Game Launcher automatically inspects local manifests and registries across all installed platform clients. See [External Platforms](External-Platforms.md) for full details on multi-launcher discovery.

## 🗂️ Organizing & Collections

Manually added local games appear seamlessly in the **Library** view alongside all your storefront titles with a platform badge (such as **Epic**, **GOG**, or **Other**). You can:
- Filter and sort them dynamically by name, playtime, or install status.
- Add them to custom collections and dynamic smart collections.
- Auto-categorize them by genre, developer, and tags via [AutoCategorization](AutoCategorization.md).

## 🛡️ Persistence & Backups

All metadata, custom tags, artwork slot assignments, and collection memberships for manually added games are persisted safely in the generic embedded SQLite database (`database.sqlite`) with automatic transactional backups.

## ➡️ Next steps

- [External Platforms](External-Platforms.md)
- [Artwork Management](Artwork-Management.md)
- [Metadata Providers](Metadata-Providers.md)
- [Collections](Collections.md)
