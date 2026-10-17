---
layout: wiki
title: Security
summary: Zero external file modification, non-invasive credential boundaries, and secrets management
parent: Technical Wiki
nav_order: 7
---

# Security

Universal Game Launcher is built on a strict **non-invasive, privacy-first security model**. It runs 100% on-device and never alters host storefront configuration files or harvests third-party user credentials.

## 🔒 Non-Invasive Invariant

Unlike legacy tools that modified store files, Universal Game Launcher is entirely decoupled:
- **Never modifies external client files:** Does not edit Steam's `shortcuts.vdf`, `cloud-storage-namespace-1.json`, `appmanifest.acf`, or external client databases.
- **No process-locking requirements:** Official platform launchers do not need to be closed; Universal Game Launcher operates concurrently without risking file corruption or sync races.
- **Local SQLite Persistence:** Custom collections, favorite flags, and artwork assignments are saved strictly in `database.sqlite` under the application's isolated local app data folder.

## 🔑 Secrets & Plugin Configuration

- **Zero Hardcoded Secrets:** No API keys or tokens are hardcoded into the source code or binary.
- **Per-Plugin Settings Isolation:** Optional API keys provided by the user (e.g. SteamGridDB, IGDB, MobyGames) are stored in the local SQLite database and provided only to the specific plugin requesting them.
- **Password Input Hygiene:** The frontend renders sensitive credential inputs with password masking and explicit eye show/hide toggles.

## 🚀 Process Execution & Launcher Delegation

- **Zero Credential Scraping:** The launcher never requests, intercepts, or stores user passwords or login credentials.
- **Safe Protocol Delegation:** Launches are executed via documented operating system protocol URIs (`steam://rungameid/{id}`, `com.epicgames.launcher://`, `goggalaxy://`) or by launching verified local executable paths specified by the user.

## 🕵️ Reporting a Vulnerability

Report suspected vulnerabilities privately through GitHub Security Advisories on the repository, or by opening a security issue. See [SECURITY.md](https://github.com/HELIX-Origin/Universal-Game-Launcher/blob/main/SECURITY.md) for the full disclosure policy.

## ➡️ Next Steps

- [Privacy Model](Privacy.md) — local data handling and zero telemetry.
- [Plugins Architecture](Plugins-Architecture.md) — plugin sandboxing and schema compliance.
