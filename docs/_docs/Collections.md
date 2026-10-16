---
layout: docs
title: Collections
summary: How Steam Vault stores and writes collections safely
parent: Documentation Home
nav_order: 4
---

# Collections

In Steam Vault, a **collection** is the same category Steam shows in your library. Steam Vault reads
your existing collections and writes edits straight back to Steam.

## 🗃️ Where collections live

Modern Steam stores collections in a cloud-synced namespace file inside your Steam user data:

```plaintext
userdata/<account-id>/config/cloudstorage/cloud-storage-namespace-1.json
```

Steam Vault reads and writes this file directly. Two built-in collections are handled specially:

- **Favorites** — the built-in `favorite` collection.
- **Hidden** — the built-in `hidden` collection.

Older Steam accounts sometimes also stored categories as tags in `localconfig.vdf`. Steam Vault can
read those as a best-effort import, but on modern accounts that list is usually empty because
collections moved to the cloud-storage file.

## ➕ Creating and editing

In the **Collections** view you can:

- **Create** a new collection by name.
- **Delete** a collection (the built-in Favorites and Hidden collections cannot be deleted).

In the **Library** view you can move selected games in and out of any collection.

## 🛡️ Write safety

Writing to Steam's files is the one genuinely sensitive operation, so Steam Vault is deliberately
cautious:

1. **Steam must be closed.** If Steam is running, the write is refused with a clear message. This
   avoids racing Steam's own cloud sync.
2. **A backup is taken first.** Before modifying a file, Steam Vault copies it to a timestamped
   backup in the app data directory.

> **Important:** Collections are cloud-synced by Steam. If you edit them on another device at the
> same time, one side can win when Steam next syncs. Closing Steam first and keeping backups makes
> this recoverable.

## 🧩 Dynamic collections

Steam supports filter-backed ("dynamic") collections that populate themselves from a filter spec.
Steam Vault detects and displays dynamic collections; authoring new dynamic filter specs from the
UI is on the roadmap.

## ➡️ Next steps

- [AutoCategorization](AutoCategorization.md) — build collections in bulk.
- [Manually Added Local Games](Non-Steam-Games.md) — organizing standalone games and custom executables.
