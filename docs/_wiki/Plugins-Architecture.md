---
layout: wiki
title: Plugins Architecture & Ecosystem
summary: Deep dive into the Universal Game Launcher plugin lifecycle, schema specs, and runtime resolution
parent: Technical Wiki
nav_order: 9
---

# Plugins Architecture & Ecosystem

Universal Game Launcher implements a modular plugin engine designed for zero-latency game discovery, multi-provider metadata enrichment, and decoupled community extensibility.

---

## Architectural Principles

1. **Decoupled Repositories**: The core launcher code (`Steam-Vault`) is completely decoupled from provider integrations (`Universal-Game-Launcher-Plugins`).
2. **Declarative Schemas**: All plugin capabilities, API routes, launch schemes, and user settings are defined statically in `plugin.schema.json`.
3. **User-Configurable Execution**: Only plugins explicitly enabled by the user in settings are invoked. Keyed plugins never transmit credentials anywhere except to their designated endpoints.
4. **Air-Gapped Local Operation**: The core launcher operates fully offline without any mandatory external services.

```
┌────────────────────────────────────────────────────────┐
│               Universal Game Launcher                  │
│                                                        │
│  ┌──────────────────┐          ┌────────────────────┐  │
│  │   PluginEngine   │─────────▶│    SQLite Cache    │  │
│  └────────┬─────────┘          └────────────────────┘  │
└───────────┼────────────────────────────────────────────┘
            │ Loads & validates manifests
            ▼
┌────────────────────────────────────────────────────────┐
│     Universal-Game-Launcher-Plugins (GitHub CDN)       │
│                                                        │
│  ├── index.json (Official Plugin Registry)             │
│  ├── schemas/plugin.schema.json                        │
│  ├── plugins/platforms/* (Steam, GOG, Epic, itch...)   │
│  └── plugins/metadata/*  (VNDB, IGDB, ProtonDB...)     │
└────────────────────────────────────────────────────────┘
```

---

## Runtime Plugin Lifecycle

1. **Registry Fetch**: On demand, `PluginEngine` queries the raw `index.json` registry from the official repository.
2. **Manifest Verification**: Each plugin manifest is fetched and validated against `schemas/plugin.schema.json`.
3. **Settings Projection**: If the plugin defines `settings.fields`, the launcher frontend renders the necessary credential inputs in the Settings page.
4. **Execution & Enrichment**:
   - **Platform plugins** scan local installation folders and dispatch launch protocols.
   - **Metadata plugins** query remote endpoints concurrently with configurable rate-limiting and cache the results in SQLite.

---

## Related Documents

- [Plugin Development Guide]({{ '/docs/Plugin-Development/' | relative_url }})
- [Metadata Providers Reference]({{ '/docs/Metadata-Providers/' | relative_url }})
- [Architecture Overview]({{ '/wiki/Architecture/' | relative_url }})
