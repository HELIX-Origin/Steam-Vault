---
layout: wiki
title: Privacy
summary: What leaves your machine, and what never does
parent: Steam Vault Technical Wiki
nav_order: 8
---

# Privacy

Steam Vault is designed to keep your data on your machine. This page is explicit about the very
small amount that ever leaves it.

## 🏠 Local by default

- All library parsing, collection editing, and categorization run **locally**.
- There is **no telemetry**, no analytics, no accounts, and no first-party server.
- Your library, playtime, and account identifiers are never uploaded to HELIX Origin.

## 🌐 What does go over the network

The only outbound requests are **metadata lookups you trigger**, and they go directly to the
provider — never through a proxy of ours:

| Destination | When | Key required |
| :--- | :--- | :--- |
| `store.steampowered.com` | Fetching Steam Store metadata | No |
| `api.steampowered.com` | Fetching owned games / summaries | Yes |
| `www.steamgriddb.com` | Searching and fetching artwork | Yes |
| `id.twitch.tv` / `api.igdb.com` | IGDB token and queries | Yes |
| `www.pcgamingwiki.com` | Supplemental metadata | No |
| `en.wikipedia.org` | Article summaries | No |

Each provider sees only what its API requires — typically a game name or app id, and your key. See
[Metadata Providers]({{ '/docs/Metadata-Providers/' | relative_url }}) for what each contributes.

## 🗄️ Local storage

- **Metadata cache** — fetched results are cached locally as JSON for speed.
- **Settings** — including API keys, stored in your user configuration directory.
- **Backups** — timestamped copies of Steam files, stored in the app data directory.

You can delete any of these at any time; the app rebuilds what it needs.

## ➡️ Next

- [Security](Security.md)
- [Metadata Providers]({{ '/docs/Metadata-Providers/' | relative_url }})
