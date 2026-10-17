---
layout: docs
title: Metadata Providers
summary: Extensive metadata provider plugins, capabilities, and user configuration
parent: Documentation Home
nav_order: 6
---

# Metadata Providers & Plugins

Universal Game Launcher connects to an extensive ecosystem of modular metadata providers. Each provider is delivered as a dedicated in-app modular plugin built directly into the application under `src/lib/plugins/metadata/`.

You can enable, disable, and configure individual providers directly from **Settings → Plugins** or through the dedicated **Plugin Settings** sub-window dialog. Only the providers you enable are queried during library scans and showcase rendering.

---

## 🔌 Provider Reference

| Provider Plugin | Key Required | Endpoint / Protocol | Capabilities & Information Provided |
| :--- | :---: | :--- | :--- |
| **Steam Store** | No | `store.steampowered.com/api/appdetails` | Descriptions, genres, release dates, features, Cloudflare CDN covers, and 1080p screenshots. |
| **GOG Catalog** | No | `embed.gog.com` | Official GOG vertical covers, hero wallpapers, and publisher information. |
| **Epic Games Store** | No | `store-content.ak.epicgames.com` | Official EGS catalog details, developer metadata, and wide keyart banners. |
| **itch.io Catalog** | No | `itch.io/api/1` | Indie title descriptions, cover images, and creator links. |
| **SteamGridDB** | User Key | `www.steamgriddb.com/api/v2` | Community-crafted vertical capsule art, hero banners, logos, and custom grid assets. |
| **IGDB (Twitch)** | User Key | `api.igdb.com/v4` | Twitch IGDB game taxonomy, franchise links, developer studios, and user ratings. |
| **VNDB** | No | `api.vndb.org/kana/vn` | Visual novel synopses, character profiles, voice cast rosters, and Japanese/English release data. |
| **IndieDB / ModDB** | No | `www.indiedb.com/games` | Independent releases, standalone mods, total conversions, and developer logs. |
| **PCGamingWiki** | No | `www.pcgamingwiki.com/w/api.php` | Save file directories, config file paths, FOV/ultrawide fixes, controller support, and community patches. |
| **Wikipedia & Wikidata** | No | `en.wikipedia.org/w/api.php` | Encyclopedic summaries, game engine specifications, director/composer credits, and historical context. |
| **MobyGames** | User Key | `api.mobygames.com/v1/games` | Historical credit registries, authentic box art scans, and platform port lineage. |
| **Giant Bomb** | User Key | `www.giantbomb.com/api/games/` | Editorial wiki decks, franchise universes, gameplay concepts, and character rosters. |
| **ProtonDB** | No | `www.protondb.com/api/v1` | Linux / Steam Deck compatibility tiers (Native, Platinum, Gold) and launch option tweaks. |
| **HowLongToBeat** | No | `howlongtobeat.com/api/search` | Main Story, Main + Extra, and Completionist playtime averages. |
| **Metacritic** | No | `fandom-prod.metacritic.com` | Critic Metascore, User Score, and "Must-Play" accolade badges. |
| **OpenCritic** | No | `api.opencritic.com/api/game` | Top Critic Average, Critics Recommend %, and tier badges (Mighty, Strong, Fair, Weak). |

---

## 🔑 User-Configurable Settings

Providers that require credentials or support custom preferences expose these directly in the launcher interface (**Settings → Plugins → [Provider Name]**).

- **SteamGridDB**: Enter your free API key from your SteamGridDB account preferences to unlock community capsule and hero artwork.
- **IGDB**: Enter your Twitch Client ID and Client Secret; Universal Game Launcher manages OAuth token acquisition automatically.
- **MobyGames**: Enter your API key from your MobyGames profile to access retro packaging scans and credits.
- **Giant Bomb**: Enter your personal API key from Giant Bomb.
- **Keyless Providers**: Function immediately without authentication.

---

## 🗄️ Local Caching & Performance

All fetched metadata is cached locally in the embedded SQLite database (`database.sqlite`). Subsequent requests are served with sub-millisecond latency. Cached entries are refreshed on a configurable schedule (default: 7 days) or manually via the **↻ Refresh Metadata** action in the game showcase.

---

## ➡️ Next Steps

- [Plugin Development Guide](Plugin-Development.md)
- [Settings & API Keys](Settings-and-API-Keys.md)
- [Collections](Collections.md)
