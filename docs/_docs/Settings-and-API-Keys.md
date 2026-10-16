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
| **Language** | Display language and metadata language. Select from all 28+ officially supported languages. See [Localization](Localization.md). |
| **Library Metadata Refresh** | One-click button to fetch and refresh metadata across all supported providers for every game in your library, with an optional toggle to also download missing vertical artwork. |
| **Back up before writing** | Take a timestamped backup of every file before it is modified. |
| **Platforms** | Configure which official launchers to detect, and add custom game-folder directories for standalone games. |
| **API keys** | Keys for the optional keyed providers (see below). |

## 🎮 Platform sources

Universal Game Launcher automatically detects games installed across official storefronts. Under **Settings → Platforms** you
turn each source (Steam, GOG, Epic, Xbox, EA, Ubisoft, Battle.net, itch.io, Amazon) on or off, add extra directories to search, and
configure multi-drive scanning. You can also specify custom game install folders for manually added local games.
See [Supported Launchers & Platforms](External-Platforms.md).

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
