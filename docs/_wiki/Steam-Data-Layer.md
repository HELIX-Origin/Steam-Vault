---
layout: wiki
title: Steam Data Layer
summary: The Steam files Steam Vault reads and writes, their formats, and write safety
parent: Steam Vault Technical Wiki
nav_order: 3
---

# Steam Data Layer

All Steam knowledge lives in `src-tauri/src/steam/`. This page records which files are involved,
what format they use, and how writes are protected.

## 📂 Files read

| File | Format | Used for |
| :--- | :--- | :--- |
| `libraryfolders.vdf` | KeyValues (text) | The list of Steam library folders and which apps live in each. |
| `appmanifest_<appid>.acf` | KeyValues (text) | Installed games per library: name, install dir, size, state. |
| `localconfig.vdf` | KeyValues (text) | Per-app play stats and, on older accounts, legacy tags. |
| `shortcuts.vdf` | KeyValues (**binary**) | Non-Steam shortcuts. |
| `loginusers.vdf` | KeyValues (text) | Account names and SteamID mapping. |
| `cloud-storage-namespace-1.json` | JSON | Modern collections (categories, favorites, hidden). |

## ✍️ Files written

Only two files are ever modified:

- `userdata/<account-id>/config/cloudstorage/cloud-storage-namespace-1.json` — collections.
- `userdata/<account-id>/config/shortcuts.vdf` — non-Steam shortcuts.

## 🔤 Parsers

Two KeyValues parsers are implemented from scratch:

- **Text KeyValues** (`vdf.rs`) — tokenizing nested `{ }` objects and quoted strings, plus a
  serializer that round-trips.
- **Binary KeyValues** (`binvdf.rs`) — the type-tagged binary format Steam uses for `shortcuts.vdf`.
  The empty-file case is exactly the bytes `00 73 68 6F 72 74 63 75 74 73 00 08 08`
  (`\0shortcuts\0`, object-open `08`, object-close `08`).

Both have unit tests, including round-trip coverage.

## ☁️ Collections format

Modern collections are stored as a JSON array of `[key, entry]` pairs. Each collection key is
`user-collections.<id>`, and its `entry.value` is an **escaped JSON string** describing
`{ id, name, added, removed }` and optionally a `filterSpec` for dynamic collections. Built-ins are
`user-collections.favorite` and `user-collections.hidden`; deleted entries carry `is_deleted: true`.

## 🛡️ Write safety

Every write path follows the same contract:

1. **Steam must be closed.** `process::is_steam_running()` gates writes; if Steam is running, the
   command returns an error instead of proceeding.
2. **Back up first.** `backup::backup_file(path)` copies the target to a timestamped file before it
   is modified.
3. **Write atomically-enough and report.** The collection store is re-serialized and saved; errors
   surface to the UI rather than being swallowed.

> **Why so careful?** Collections are cloud-synced by Steam. Editing them while Steam is running, or
> concurrently on another device, can create a sync conflict. Closing Steam first and keeping
> backups makes any conflict recoverable.

## ➡️ Next

- [Backend (Rust)](Backend-Rust.md)
- [Security](Security.md)
