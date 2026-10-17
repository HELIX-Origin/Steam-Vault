---
layout: wiki
title: Privacy
summary: What leaves your machine, and what never does
parent: Universal Game Launcher Technical Wiki
nav_order: 8
---

# Privacy

Universal Game Launcher is engineered to keep your gaming data strictly on your machine. This document explicitly outlines what is processed locally and what minimal network activity may occur.

---

## 🏠 Local by Default

- All library discovery, collection organization, categorizations, and playtime calculations run **100% locally on your machine**.
- There is **no telemetry**, no usage tracking, no remote logging, no user account system, and no first-party telemetry server.
- Your gaming library, installed paths, playtime records, and account identifiers are **never uploaded to HELIX Origin** or any central cloud service.

---

## 🌐 Network Communication

Universal Game Launcher operates fully offline. The only outbound HTTP requests that ever occur are **metadata and artwork lookups you trigger**, and they connect directly to the respective third-party provider — never through an intermediary proxy:

| Destination | When Triggered | API Key Required |
| :--- | :--- | :--- |
| `store.steampowered.com` | Fetching official Steam Store details & CDN artwork | ❌ No |
| `embed.gog.com` | Fetching official GOG catalog descriptions & covers | ❌ No |
| `store-content.ak.epicgames.com` | Fetching Epic Games Store catalog manifests & keyart | ❌ No |
| `itch.io/api` | Fetching itch.io indie game details & creator profiles | ❌ No |
| `api.thegamesdb.net` | Fetching retro console ROM metadata & authentic box art | ❌ No (Optional dev key) |
| `screenscraper.fr` | Fetching multi-console ROM box art, wheel logos & video snaps | ❌ No |
| `howlongtobeat.com` | Fetching community gameplay completion times | ❌ No |
| `www.pcgamingwiki.com` | Fetching game engine, FOV/widescreen fixes, and save locations | ❌ No |
| `www.protondb.com` | Fetching Linux & Steam Deck compatibility ratings | ❌ No |
| `www.gamerpower.com` | Fetching 100% free game giveaways, promotions, and patch notes | ❌ No |
| `api.steamgriddb.com` | Fetching community capsule covers, heroes, logos, and square icons | ✅ Yes (Optional) |
| `id.twitch.tv` / `api.igdb.com` | Fetching IGDB game taxonomy, franchise links, and critic ratings | ✅ Yes (Optional) |

Each provider endpoint receives only what its specific API query requires — typically a game title string or numeric app ID, along with your user-configured API key (if keyed).

---

## 🗄️ Local Persistence

- **Local SQLite Database (`database.sqlite`)**: Persists custom collections, user title overrides, favorite flags, and cached metadata records locally on your system.
- **In-Memory KV Cache (`mem_kv`)**: Sub-millisecond transient key-value cache persisted to disk on shutdown.
- **Settings (`settings.json`)**: Application preferences and localized UI selections stored in your user configuration directory.
- **Backups (`backups/`)**: Timestamped backups taken prior to physical file relocations.

You can purge or reset the metadata and artwork cache at any time under **Settings → Local Storage & Cache Architecture**.

---

## ➡️ Next Steps

- [Security Model](Security.md) — non-invasive invariants and credential boundaries.
- [Metadata Providers Reference]({{ '/docs/Metadata-Providers/' | relative_url }}) — provider details and capabilities.
