---
layout: docs
title: Artwork Management
summary: Applying grid, hero, logo, and icon art to any game, including non-Steam shortcuts
parent: Documentation Home
nav_order: 13
---

# Artwork Management

The **Artwork** view lets you replace the artwork Steam shows for any game in your library — Steam
games and non-Steam shortcuts alike. You can pull art from **SteamGridDB** or apply your own image
file, and you can clear art you no longer want.

## 🖼️ The five slots

Steam stores custom artwork per game in five **slots**. Each slot is a fixed filename and a typical
size:

| Slot | File name | Usual size |
| :--- | :--- | :--- |
| **Portrait grid** | `<appid>p` | 600×900 |
| **Landscape grid** | `<appid>` | 920×430 |
| **Hero banner** | `<appid>_hero` | 1920×620 |
| **Logo** | `<appid>_logo` | transparent PNG, any size |
| **Icon** | `<appid>_icon` | 256×256 |

Steam Vault writes the image with an extension matching its real format (`.png`, `.jpg`, or
`.jpeg`), so Steam always loads it.

## 📂 Where the files live

Custom artwork is stored inside your Steam user data:

```plaintext
userdata/<account-id>/config/grid/
```

This is the same location Steam's own **Customize** screen writes to. Because it is keyed by app id,
the same system covers **non-Steam shortcuts**, whose synthetic app ids have no store page of their
own.

## 🌐 Applying art from SteamGridDB

1. Open **Artwork** and pick a game from the list on the left.
2. Next to the slot you want, choose **Browse…**.
3. SteamGridDB results appear as thumbnails; click one to apply it.

SteamGridDB requires an API key — add one in [Settings & API Keys](Settings-and-API-Keys.md). Without
a key you can still apply art from a local file.

**How matching works:** for a real Steam game Steam Vault looks artwork up by its **exact app id**
(so you get the right game, not a search result). For a **non-Steam shortcut** — which has no store
page — it searches SteamGridDB by name and prefers an exact title match.

## ⚡ Automatic Artwork Fetching

Steam Vault automatically ensures games don't have blank gray posters in your Steam library:

- **Automatic on Import:** When scanning and importing non-Steam games, ROMs, or platform titles from GOG/Epic/itch, Steam Vault automatically attempts to download missing artwork (portrait grid, hero banner, logo, and icon) via SteamGridDB if an API key is configured. If no key is set or no match is found, it falls back to extracting cover art from Steam Store, VNDB, or itch metadata caches.
- **Auto-fetch Missing Art Button:** In the **Artwork** view header, click **Auto-fetch missing art** to run a background pass across your entire library and populate all games that currently lack grid art.

## 📁 Applying art from a file

1. Select the game and the slot, then choose **From file…**.
2. Paste the full path to a PNG or JPEG image and click **Apply**.

Only **PNG** and **JPEG** images are accepted, up to **64 MiB**. Anything else is rejected with a
message, because Steam cannot display it.

## 🗑️ Removing art

If a slot already has custom art, its card shows a **Remove** button. The previous image is copied to
the timestamped backup store first, so you can always restore it by hand.

## 🛡️ Write safety

- **Steam does not need to be closed** to change artwork — Steam watches the `config/grid` folder and
  repaints with the new image. (Collection and shortcut writes still require Steam to be closed.)
- Before an image is replaced or removed, the old one is copied into
  `steam-vault/backups/<timestamp>/`. Nothing is destroyed irreversibly.
- The Artwork view never touches a game's install files — it only manages the images in
  `config/grid`.

## 💡 Tips

- Portrait grids and heroes give the biggest visual improvement for non-Steam entries.
- A **logo** should be a transparent PNG, or it will show a coloured box behind it.
- If Steam does not immediately show new art, switch to another library tab and back, or restart
  Steam.

## ➡️ Next steps

- [Non-Steam Games](Non-Steam-Games.md)
- [Metadata Providers](Metadata-Providers.md)
- [Storage Management](Storage-Management.md)
