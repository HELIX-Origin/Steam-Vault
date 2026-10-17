---
layout: docs
title: Storage Management
summary: Inspecting drives, volume utilization, and managing multi-platform game storage
parent: Documentation Home
nav_order: 12
---

# Storage Management

The **Storage** view (`/storage`) in Universal Game Launcher provides a comprehensive storage analytics cockpit. It reveals how your installed games are distributed across physical drives, mounted volumes, and storefront library directories.

---

## ⚡ Non-Blocking Background Worker & Plugin Gating

The Storage view is powered by the **Storage & Drive Analyzer** utility plugin (`utility-storage`):
- **Instant UI Rendering**: Switching to the Storage tab renders the page structure immediately with 0ms blocking time. Heavy disk inspection is dispatched to a background thread pool (`tauri::async_runtime::spawn_blocking`), streaming metrics in reactively without UI hitches.
- **Plugin-Aware Filtering**: Only installed titles from enabled platform plugins (`platform-steam`, `platform-gog`, `platform-epic`, etc.) are tracked and computed. Disabling a storefront plugin in **Settings → Plugins** automatically excludes its titles from the storage footprint.
- **Utility Plugin Control**: If `utility-storage` is disabled in plugin settings, drive scanning is deactivated gracefully.

---

## 💾 Multi-Drive Capacity Gauges

At the top of the Storage view, Universal Game Launcher visualizes disk utilization across every mounted drive on your system:

- **Drive Capacity Gauges**: Visual percentage bars indicating used vs. available gigabytes.
- **Game Footprint vs. Other Data**: Differentiates between storage consumed by discovered games and other operating system or user data.
- **Drive Health & Threshold Alerts**: Clear visual warnings when volumes are running low on disk space.

---

## 🗂️ Library Volumes & Platforms

Below the system drive gauges, games are grouped and filterable by platform origin and storage volume:

| Platform | Storage Discovery Mechanism |
| :--- | :--- |
| **Steam** | Parses all registered libraries in `libraryfolders.vdf` and game manifests (`appmanifest_*.acf`). |
| **GOG Galaxy** | Queries local Galaxy SQLite DB and install paths for discrete game directory sizes. |
| **Epic Games** | Reads `.item` manifests in `Manifests/` for install sizes and target directories. |
| **Xbox / Windows Apps** | Inspects AppX/MSIX package footprints across system and secondary drives. |
| **EA Desktop & Ubisoft Connect** | Inspects installer registry entries and configured game folders. |
| **Battle.net & Amazon Games** | Reads client product databases and manifests. |
| **Standalone / ROMs** | Directly calculates disk usage of manually added executables, directories, and emulator ROMs. |

---

## 🎮 Game Storage Explorer

- **Search & Filter**: Search games by name or filter to titles installed on a specific drive.
- **Sort by Size**: Instantly sort from largest to smallest to identify games consuming the most disk space.
- **One-Click Folder Access**: Click the **Open Folder** action to open the game's local installation directory directly in your operating system's file manager (Windows Explorer, macOS Finder, or Linux file manager).
- **Uninstall Protocol Trigger**: Initiate uninstallation via native client protocol handlers (`steam://uninstall/{appId}`, `com.epicgames.launcher://apps/{appId}?action=uninstall`) or official launcher delegation.

---

## 🛡️ Non-Invasive Storage Safety

Universal Game Launcher operates completely read-only against external client configuration files:
- It **never modifies** Steam's `libraryfolders.vdf` or client configuration databases.
- Relocating games between drives or managing game installations is performed through official client interfaces or your operating system, preventing cloud synchronization issues and file locking conflicts.

---

## ➡️ Next Steps

- [Organizing Your Library](Organizing-Your-Library.md) — filter by installed games in the left sidebar.
- [Manually Added Local Games](Non-Steam-Games.md) — manage standalone game paths.
- [Troubleshooting](Troubleshooting.md) — resolving storage and permission issues.
