---
layout: wiki
title: Architecture
summary: The big picture and module map of Steam Vault
parent: Steam Vault Technical Wiki
nav_order: 2
---

# Architecture

Steam Vault is a single desktop process with two halves: a **Rust / Tauri v2 backend** that owns all
Steam data access, and a **SvelteKit static SPA** that renders the interface and calls the backend
over typed IPC.

```mermaid
flowchart TB
    ui[SvelteKit SPA] --> ipc[Typed Tauri commands]
    ipc --> core[Rust core: library, collections, storage, autocat, filter]
    core --> steam[Steam data layer]
    core --> meta[Metadata providers]
    steam --> disk[(Steam local files)]
    meta --> cache[(SQLite metadata cache)]
    core --> backup[(Timestamped backups)]
```

The design rule is simple: **the frontend never touches the filesystem**. Every read and write goes
through a Rust command, which is where validation, Steam-closed checks, and backups live.

## 🧱 Module map

The backend is split into focused modules:

| Module | Responsibility |
| :--- | :--- |
| `steam/` | Locating Steam, parsing Valve KeyValues (text and binary), reading libraries, collections, and shortcuts, and moving installed games between library folders. |
| `metadata/` | One client per provider, plus the shared HTTP client and provider registry. |
| `autocat` | The rule engine that turns metadata into proposed collections. |
| `filter` | The Allowed / Required / Excluded filter model. |
| `cache` | Embedded SQLite database (`metadata.db`) with WAL mode, indexing, TTL checking, and precompiled Community Game Catalog seed (`catalog_seed.bin` / `catalog.db`). |
| `settings` | Loading and saving local settings and API keys. |
| `backup` | Timestamped file backups taken before any write. |
| `commands` | The Tauri IPC surface and shared application state. |

## 🔁 Typical flow

```mermaid
sequenceDiagram
    participant UI as SvelteKit SPA
    participant CMD as Tauri command
    participant Core as Rust core
    participant Steam as Steam files
    UI->>CMD: load_library()
    CMD->>Core: parse manifests + collections + shortcuts
    Core->>Steam: read appmanifest_*.acf, namespace json, shortcuts.vdf
    Steam-->>Core: raw data
    Core-->>UI: Vec<Game> (camelCase JSON)
```

For writes, the same path applies with two extra guards: the app checks that Steam is not running,
and it backs up the target file before modifying it.

## 🎯 Design goals

- **Resilience** — read documented local formats and public APIs, never scrape HTML.
- **Safety** — refuse risky writes and make every write reversible.
- **Clarity** — keep Steam knowledge in one layer and the UI free of filesystem concerns.

## ➡️ Next

- [Steam Data Layer](Steam-Data-Layer.md)
- [Backend (Rust)](Backend-Rust.md)
