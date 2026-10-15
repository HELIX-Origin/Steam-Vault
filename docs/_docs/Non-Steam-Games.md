---
layout: docs
title: Non-Steam Games
summary: Importing and organizing non-Steam shortcuts with artwork
parent: Documentation Home
nav_order: 7
---

# Non-Steam Games

Steam lets you add games that are not in the Steam store as **shortcuts**. Steam Vault reads and
writes those shortcuts so you can organize them alongside your Steam games.

## 📄 Where shortcuts live

Non-Steam shortcuts are stored in Steam's binary KeyValues file:

```plaintext
userdata/<account-id>/config/shortcuts.vdf
```

Steam Vault parses this binary format directly, so it does not depend on Steam's UI.

## 🖼️ Artwork with SteamGridDB

Non-Steam games rarely have official artwork, so Steam Vault uses **SteamGridDB** to find grids
(covers), heroes (banners), logos, and icons. This is the main reason SteamGridDB support matters:
it lets non-Steam entries look at home in your library. The dedicated
[Artwork Management](Artwork-Management.md) view applies those images to a game's slots — or applies
your own image files.

> **Note:** SteamGridDB requires an API key. Add one in
> [Settings & API Keys](Settings-and-API-Keys.md) to enable artwork lookups.

## 🎮 Importing from other launchers

Rather than adding each game by hand, use the **Platforms** view to detect installed games from GOG,
Epic Games, itch.io, Amazon Games, Xbox / Microsoft Store, Ubisoft Connect, EA, Battle.net, and
Rockstar, and import them in bulk. See [External Platforms](External-Platforms.md).

## 🗂️ Organizing shortcuts

Non-Steam shortcuts appear in the **Library** view with a pill naming their source platform (for
example **GOG** or **itch.io**). You can re-tag a shortcut's source storefront directly in the
Library (individually or in bulk), add and remove them from collections exactly like Steam games,
and the **Organize** view can group them by **Platform** in a single step.

## 🛡️ Write safety

Shortcut writes follow the same rules as collection writes: **Steam must be closed**, and a
timestamped backup of `shortcuts.vdf` is taken before any change.

## ➡️ Next steps

- [External Platforms](External-Platforms.md)
- [Metadata Providers](Metadata-Providers.md)
- [Collections](Collections.md)
