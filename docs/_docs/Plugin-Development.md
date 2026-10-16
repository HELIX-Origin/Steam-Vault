---
layout: docs
title: Plugin Development Guide
summary: How to author, test, and publish platform integrations and metadata plugins
parent: Documentation Home
nav_order: 14
---

# Plugin Development Guide

Universal Game Launcher provides a modular plugin ecosystem enabling community developers to author custom platform integrations and metadata enrichment providers.

Official plugins and formal schema definitions are hosted in the [Universal-Game-Launcher-Plugins](https://github.com/HELIX-Origin/Universal-Game-Launcher-Plugins) repository.

---

## Plugin Architecture

All plugins are self-contained packages containing:
1. `index.json`: Declarative JSON manifest conforming to `schemas/plugin.schema.json`.
2. `README.md`: Documentation detailing setup, supported features, and required credentials.

```
my-plugin/
├── index.json      # Declarative manifest
└── README.md       # Human-readable instructions
```

---

## Authoring a Metadata Provider Plugin

Metadata provider plugins enrich games with covers, heroes, descriptions, ratings, and technical details.

### Example Manifest (`index.json`)

```json
{
  "$schema": "https://raw.githubusercontent.com/HELIX-Origin/Universal-Game-Launcher-Plugins/main/schemas/plugin.schema.json",
  "id": "metadata-mycommunity",
  "name": "My Community Database",
  "version": "1.0.0",
  "type": "metadata",
  "author": "Community Developer",
  "description": "Enriches titles with custom artwork and reviews from My Community Database.",
  "homepage": "https://github.com/my-org/my-plugin",
  "features": ["capsule_artwork", "critic_reviews"],
  "metadata": {
    "apiUrl": "https://api.mycommunity.org/v1/games"
  },
  "settings": {
    "requiresApiKey": true,
    "fields": [
      {
        "key": "apiKey",
        "label": "API Key",
        "type": "password",
        "description": "Your API key from https://mycommunity.org/api.",
        "required": true
      }
    ]
  }
}
```

---

## Authoring a Platform Integration Plugin

Platform integration plugins enable library scanning, manifest parsing, and game launching for custom stores and launchers.

### Example Manifest (`index.json`)

```json
{
  "$schema": "https://raw.githubusercontent.com/HELIX-Origin/Universal-Game-Launcher-Plugins/main/schemas/plugin.schema.json",
  "id": "platform-mylauncher",
  "name": "My Custom Launcher Integration",
  "version": "1.0.0",
  "type": "platform",
  "author": "Community Developer",
  "description": "Discovers and launches titles installed via My Launcher.",
  "features": ["discovery", "launch"],
  "platform": {
    "platformId": "mylauncher",
    "launchProtocols": ["mylauncher://launch/{appId}"],
    "knownFolders": [
      "%ProgramFiles%/MyLauncher/Games",
      "%LOCALAPPDATA%/MyLauncher/Library"
    ],
    "executableNames": ["MyLauncher.exe"]
  }
}
```

---

## Testing & Schema Validation

To validate your plugin manifest before publishing:

```bash
git clone https://github.com/HELIX-Origin/Universal-Game-Launcher-Plugins.git
cd Universal-Game-Launcher-Plugins
npm run validate
```

The validator verifies:
- Strict JSON syntax and schema compliance.
- Naming conventions (`metadata-*`, `platform-*`).
- Presence of required fields (`id`, `name`, `version`, `type`).
- Documentation completeness (`README.md`).

---

## Publishing Your Plugin

1. Fork the official [Universal-Game-Launcher-Plugins](https://github.com/HELIX-Origin/Universal-Game-Launcher-Plugins) repository.
2. Place your plugin under `plugins/platforms/<slug>` or `plugins/metadata/<slug>`.
3. Add your entry to root `index.json`.
4. Ensure `npm run validate` passes with zero errors.
5. Open a Pull Request. Once merged, Universal Game Launcher users worldwide can discover and install your plugin!
