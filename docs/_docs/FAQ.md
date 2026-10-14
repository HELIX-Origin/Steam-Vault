---
layout: docs
title: FAQ
summary: Frequently asked questions about Steam Vault
parent: Documentation Home
nav_order: 10
---

# FAQ

## Is Steam Vault safe to use on my real library?

Yes, within a clear safety model: writes are refused while Steam is running, and every modified file
is backed up with a timestamp first. You can always restore a backup. See
[Collections](Collections.md).

## Does it upload my data anywhere?

No. Everything is processed locally. The only network requests are metadata lookups to the providers
you enable, and only when you ask for them. See
[Metadata Providers](Metadata-Providers.md).

## Do I need API keys to use it?

No. The core organizing features work with keyless Steam Store metadata. Keys only add extra
sources: Steam Web API (owned games/playtime), SteamGridDB (artwork), and IGDB (rich metadata).

## Why is RAWG not supported?

RAWG is defunct. No RAWG provider exists, by design. Steam Web API, SteamGridDB, and IGDB are the
only keyed providers.

## Will a Steam update break it?

Steam Vault deliberately reads **stable, documented data sources** — Steam's own local files and
public APIs — rather than scraping HTML, so ordinary updates are unlikely to break it. When Steam
does change a format, the parser is isolated and can be updated in one place.

## Does it work on Linux and macOS?

Steam Vault is built with Tauri, which is cross-platform. Windows is the primary development target;
the data layer includes cross-platform discovery paths. Platform-specific testing is ongoing.

## Can I undo a change?

Yes — restore the relevant timestamped backup from the app data directory. Because Steam Vault edits
Steam's own files, a backup is the cleanest way to roll back.

## Is this affiliated with Depressurizer or Valve?

No. Steam Vault is an independent, clean-room project inspired by Depressurizer (by Andrew Martin
and contributors). It is not affiliated with or endorsed by Valve or the Depressurizer project.

## Where do I report bugs or request features?

Open an issue on the
[tracker](https://github.com/HELIX-Origin/Steam-Vault/issues). For security reports, follow
[SECURITY.md](https://github.com/HELIX-Origin/Steam-Vault/blob/main/SECURITY.md).
