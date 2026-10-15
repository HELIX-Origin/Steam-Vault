---
layout: wiki
title: Security
summary: Secrets handling, local writes, and vulnerability reporting
parent: Steam Vault Technical Wiki
nav_order: 7
---

# Security

Steam Vault runs entirely on your machine and interacts with files Steam already owns. This page
summarizes its security posture; the canonical policy lives in the repository's `SECURITY.md`.

## 🔑 Secrets & API keys

- No credentials are ever hardcoded in the source.
- Keys you provide (Steam Web API, SteamGridDB, IGDB/Twitch) are stored **only** in your local
  configuration directory and are sent **only** to the provider they belong to.
- Keys are stored as plain text in the local settings file — the same threat model as Steam's own
  configuration. Protect your user account accordingly.

## ✍️ Local writes

Steam Vault modifies only these local files:

- the collection cloud-storage namespace (JSON),
- `shortcuts.vdf` (binary KeyValues),
- `libraryfolders.vdf` (every copy), when a library is registered/removed or a game moves, and
- `appmanifest_<appid>.acf`, moved alongside its game during a storage move.

Every write is gated and reversible:

1. Writes are **refused while Steam is running**, preventing sync races.
2. A **timestamped backup** is taken before the file is modified, so changes can be undone by
   restoring the backup.

## 🕵️ Reporting a vulnerability

Report suspected vulnerabilities privately through GitHub Security Advisories on the repository, or
by opening a minimal issue that does not disclose exploit details.

See [SECURITY.md](https://github.com/HELIX-Origin/Steam-Vault/blob/main/SECURITY.md) for the full
policy.

## ➡️ Next

- [Privacy](Privacy.md)
- [Steam Data Layer](Steam-Data-Layer.md)
