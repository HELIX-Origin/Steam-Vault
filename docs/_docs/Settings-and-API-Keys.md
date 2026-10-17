---
layout: docs
title: Settings & Plugin Configuration
summary: Configuration reference for Universal Game Launcher and the tabbed Plugin Settings system
parent: Documentation Home
nav_order: 9
---

# Settings & Plugin Configuration

Universal Game Launcher separates core application preferences from individual plugin configurations. All settings are stored locally on your machine with zero cloud telemetry.

---

## ⚙️ General Settings

The **Settings** view (`/settings`) configures core launcher behavior, display options, and persistence:

| Setting | Description |
| :--- | :--- |
| **System Language** | Display and metadata language. Select from 28+ officially supported languages or auto-detect from system locale. See [Localization](Localization.md). |
| **Store Country** | Two-letter country/region code (e.g. `us`, `gb`, `de`) used for regional store pricing, currency, and age ratings. |
| **Library Metadata Refresh** | One-click button to trigger a background metadata and cover art refresh across all library titles. |
| **Cache Management** | Clear the in-memory KV cache and cached artwork without touching game installations or custom tags. |
| **Plugins & Modules Tab** | Inspect, enable, disable, and configure native plugins organized by category (`platforms`, `metadata`, `utility`, `media`). |

---

## 🧩 Dedicated Plugin Settings Dialog

To maintain a clean and modular architecture, third-party API keys and platform-specific configurations are **decoupled from general settings**. Instead, each enabled plugin that declares configurable fields renders inside the **Plugin Settings** sub-window dialog.

You can open this dialog by clicking **Open Plugin Settings** or the gear icon (`⚙`) next to any configurable plugin in the Plugins list.

### Features of the Plugin Settings Dialog

- **Tabbed Plugin Navigation**: A clean left-hand rail lists every enabled plugin with settings (Steam, GOG, Epic, SteamGridDB, IGDB, itch.io, etc.).
- **Dynamic Field Schemas**: Renders inputs based on each plugin's `PluginSettingsConfig`:
  - `text` / `string`: General configuration values.
  - `password`: API keys and secrets with a visibility eye toggle (`👁`).
  - `boolean`: Feature toggles (e.g. Allow NSFW artwork, Enable humor tags).
  - `select`: Dropdown options (e.g. Preferred artwork dimensions).
  - `path`: Executable or directory pickers.
- **Test Connection (`⚡ Test Connection`)**: Verify your API key or client path in real time before saving. The dialog reports instant success or detailed error feedback.
- **Safe Persistence**: Settings are persisted to the in-memory KV store (`plugin_cfg:<id>`) backed by `database.sqlite`, with automatic fallback migration from legacy settings.

---

## 🔑 Common Plugin API Keys

Universal Game Launcher works **100% keyless** out of the box. API keys are entirely optional and only needed if you wish to query specific third-party APIs:

| Plugin | Key Type | Where to Obtain |
| :--- | :--- | :--- |
| **SteamGridDB** (`metadata-steamgriddb`) | API Key | `steamgriddb.com` → Preferences → API |
| **IGDB** (`metadata-igdb`) | Client ID & Client Secret | Twitch Developer Console (`dev.twitch.tv/console`) |
| **Steam Web API** (`metadata-steam`) | Web API Key | `steamcommunity.com/dev/apikey` |
| **itch.io** (`metadata-itch`) | API Key | `itch.io` → Account Settings → API Keys |

All other built-in providers — including Steam Store (`appdetails`), GOG Catalog, PCGamingWiki, Wikipedia, VNDB, and ProtonDB — require **zero API keys or configuration**.

---

## 🔐 Storage & Credential Security

- **Local Storage**: Plugin configurations and API keys are stored locally on your machine in generic SQLite (`database.sqlite`) via the in-memory KV tier (`plugin_cfg:<plugin_id>`).
- **No Cloud Telemetry**: Keys and personal settings are **never** committed, never uploaded to HELIX Origin servers, and never sent to any third party other than the official provider API endpoint.
- **Threat Model**: Keys are stored as plain text locally on your disk, consistent with standard desktop application threat models (e.g. Steam's `loginusers.vdf`). Keep your user machine account secure.

---

## ➡️ Next Steps

- [Plugin Development Guide](Plugin-Development.md) — authoring plugins with custom settings schemas.
- [Metadata Providers](Metadata-Providers.md) — details on what each provider fetches.
- [Supported Launchers & Platforms](External-Platforms.md) — platform discovery options.
