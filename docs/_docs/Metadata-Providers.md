---
layout: docs
title: Metadata Providers
summary: Every metadata source, what it provides, and whether it needs a key
parent: Documentation Home
nav_order: 6
---

# Metadata Providers

Metadata powers AutoCats, the filter builder, and artwork for any game. Steam Vault
combines several sources, and only three of them require an API key.

## 🔌 Provider reference

| Provider | Key | Endpoint | Provides |
| :--- | :--- | :--- | :--- |
| **Community Game Catalog** | No | *Local seed* & GitHub Releases | Curated offline catalog, multi-platform OS (`windows`, `macos`, `linux`), console hardware, and canonical metadata |
| **Steam Store** | No | `store.steampowered.com/api/appdetails` | Genres, developers, publishers, release date, feature flags |
| **Steam Store Search** | No | `store.steampowered.com/api/storesearch` | Search non-Steam & indie titles to match official Steam app IDs |
| **Steam Web API** | Yes | `IPlayerService/GetOwnedGames` | Owned games, playtime, player summaries |
| **SteamGridDB** | Yes | `www.steamgriddb.com/api/v2` | Grids, heroes, logos, icons |
| **IGDB (Twitch)** | Yes | `api.igdb.com/v4` | Summary, genres, companies, cover |
| **VNDB** | No | `api.vndb.org/kana/vn` | Visual novel developers, publishers, release dates, tags, covers |
| **PCGamingWiki** | No | `www.pcgamingwiki.com/w/api.php` | Engine, developers, publishers, release dates |
| **Local Game Sniffer** | No | *Local disk* | itch `receipt.json`, Ren'Py signatures, RPG Maker data, PE VersionInfo |
| **Wikipedia** | No | `en.wikipedia.org/w/api.php` | Infobox wikitext parser: developers, publishers, genres, release year, console platforms, and summaries |

> **Note:** **RAWG is not supported.** The service is defunct and no RAWG provider exists by design.

## 🔑 Keyed vs keyless

- **Keyless providers** work immediately. They are best-effort supplements and may change shape; a
  failure is never fatal.
- **Keyed providers** unlock extra data once you add a key in
  [Settings & API Keys](Settings-and-API-Keys.md):
  - **Steam Web API** — needs a key; `GetOwnedGames` also requires your profile's game details to
    be public, or Steam returns `403`.
  - **SteamGridDB** — needs an API key; powers the [Artwork Management](Artwork-Management.md)
    view. Real Steam games are looked up by their **exact app id**; non-Steam shortcuts, which have
    no store page, are looked up by **name** (with an exact-title preference).
  - **IGDB** — needs a Twitch *client id* and *client secret*; Steam Vault obtains an app access
    token automatically via the client-credentials flow.

## 🗄️ Caching

Fetched metadata is cached **locally** in an embedded SQLite database (`metadata.db`) so repeat lookups are instant and API usage stays
low. Steam Store results are treated as fresh for about **7 days**; after that they are refetched.

## 🔒 Privacy

Metadata requests go only to the providers above, and only when you ask for them. Your API keys are
stored locally and are never sent anywhere except to the provider they belong to. See
[Privacy]({{ '/wiki/Privacy/' | relative_url }}) in the technical wiki.

## ➡️ Next steps

- [Settings & API Keys](Settings-and-API-Keys.md)
- [AutoCategorization](AutoCategorization.md)
