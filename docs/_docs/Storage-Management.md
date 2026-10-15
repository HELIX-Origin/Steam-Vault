---
layout: docs
title: Storage Management
summary: Inspecting libraries and drives, and moving games between them
parent: Documentation Home
nav_order: 12
---

# Storage Management

The **Storage** view is where Steam Vault shows how your installed games are spread across drives
and Steam libraries, and where you move them around.

A Steam installation keeps its games in one or more **library folders**. Each library folder is a
directory containing a `steamapps` subfolder, and the set of libraries is registered in
`libraryfolders.vdf`. Steam rewrites every copy of that file it finds on startup, so when you move
something, **every copy must be updated** or Steam will "lose" the game and re-download it.

## What the view shows

- **Drives** — every mounted disk with its used/free space, so you can see at a glance which drive
  has room.
- **Libraries** — each registered library folder with its path, whether it currently exists, how
  many installed games it holds, their total size, and the free space on its drive.
- **Games** — every installed game, searchable and filterable by library, with its size and the
  library that holds it.

## Registering and removing libraries

- **Add** — enter a folder path and Steam Vault creates its `steamapps` subfolder if needed, then
  registers it in `libraryfolders.vdf`. A library cannot live inside the Steam installation folder.
- **Remove** — unregisters a library from `libraryfolders.vdf`. The **files on disk are left
  untouched**, and a library that still holds installed games cannot be removed — move its games
  out first. The Steam installation's own library cannot be removed.

## Moving games and whole libraries

Select **Move…** on a game (or a library) to start a move:

1. **Choose a destination** — another registered library for a game, or a new path for a whole
   library.
2. **Preview** — Steam Vault reports how many bytes will move, whether the destination has enough
   free space, and whether the move stays on one drive. A same-drive move is an instant rename; a
   cross-drive move copies the files and then removes the originals.
3. **Move** — a progress bar tracks the copy. When it finishes, Steam Vault updates every copy of
   `libraryfolders.vdf` (and, for a game, moves its `appmanifest_*.acf` and any Workshop manifest
   too) so Steam finds everything in its new home.

## Safety

- **Steam must be closed.** Every change is refused while Steam is running — if it were open, Steam
  would overwrite the changes on exit or start re-downloading. Close Steam completely first.
- **Backups.** Each `libraryfolders.vdf` is backed up before it is rewritten.
- **Pre-flight checks.** Moves are validated first: the destination must have room (plus a safety
  margin), cannot be inside the source, and cannot already hold the content.

> Moving a library is a large, real file operation — it copies the actual game data. Make sure the
> destination drive is connected and has space before you start.
