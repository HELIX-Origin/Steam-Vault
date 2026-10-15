---
layout: docs
title: Settings & API Keys
summary: Configuration reference for Steam Vault
parent: Documentation Home
nav_order: 9
---

# Settings & API Keys

All configuration lives in **Settings**. Values are stored locally on your machine.

## ⚙️ Options

| Setting | Purpose |
| :--- | :--- |
| **Steam folder** | Path to your Steam installation. Use **Auto-detect** to fill it automatically. |
| **Account** | Which Steam account to organize. Leave it on *Automatic* to use the first account found. |
| **Store country** | Two-letter region used for Steam Store metadata (for example `us`). |
| **Language** | Display language and metadata language. Select from all 30 officially supported Steam languages. See [Localization](Localization.md). |
| **Community Game Catalog** | Embedded offline database of curated multi-platform games. Supports one-click online synchronization from GitHub Releases or custom URLs, plus export/import archive utilities. |
| **Back up before writing** | Take a timestamped backup of every file before it is modified. |
| **Non-Steam sources** | Which other launchers to scan for installed games, and extra config / game-folder locations to search. |
| **API keys** | Keys for the three keyed providers (see below). |

## 🎮 Non-Steam sources

Steam Vault can find games installed by other launchers. Under **Settings → Non-Steam sources** you
turn each source (GOG, Epic, itch.io, and others) on or off, add extra locations to search, and
choose whether to scan other mounted drives. Because each launcher records its games in its own
config — just like Steam's `libraryfolders.vdf` — **Find folders** autofills that launcher's config
and library locations; adding game install folders covers configs that are missing or wrong, and
local games with no launcher. See [External Platforms](External-Platforms.md).

## 🔑 API keys

| Key | Provider | Where to get it |
| :--- | :--- | :--- |
| **Steam Web API key** | Steam Web API | `steamcommunity.com/dev/apikey` |
| **SteamGridDB key** | SteamGridDB | `steamgriddb.com` account settings |
| **IGDB client id / secret** | IGDB (Twitch) | Twitch developer console |

Only these three providers require keys. Everything else — Steam Store, PCGamingWiki, and
Wikipedia — works with no configuration.

## 🔐 Where keys are stored

API keys are written to a local settings file in your user configuration directory (for example
`<config>/steam-vault/settings.json`). They are **never** committed to the repository and are never
transmitted anywhere except to the provider they belong to.

> **Note:** Keys are stored as plain text in your local config file, which is the same threat model
> as Steam's own `loginusers.vdf`. Keep your machine account secure.

## 🔄 Reverting changes

Because every write is backed up, you can restore a previous state by copying a backup from the app
data directory over the live file. See [Collections](Collections.md) for what gets written.

## ➡️ Next steps

- [Metadata Providers](Metadata-Providers.md)
- [Troubleshooting](Troubleshooting.md)
