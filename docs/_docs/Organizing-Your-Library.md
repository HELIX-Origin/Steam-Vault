---
layout: docs
title: Organizing Your Library
summary: The Library view, searching, filtering, and multi-select
parent: Documentation Home
nav_order: 3
---

# Organizing Your Library

The **Library** view is where you inspect your games and move them between collections.

## 🔎 Finding games

Three controls narrow the list:

- **Search** — filters by game name as you type.
- **Collection filter** — show only games that belong to a chosen collection.
- **Installed only** — hide entries that are not currently installed.

The header reports how many games you **own**, how many are **installed**, how many are shown, and
how many are selected.

> **The whole library, not just what is installed.** With a Steam Web API key configured, press
> **Sync owned games** to fetch every game on your account. The Library then includes games you own
> but have not installed — marked with a **not installed** pill — so you can organize your entire
> collection.

## 🖱️ Selecting and editing

1. Click a game row to select or deselect it. The checkbox on the left reflects its state.
2. Pick a **target collection** in the toolbar.
3. Use **Add** to place every selected game into that collection, or **Remove** to take them out.

Selections persist while you adjust the filters, so you can gather games across several searches
before applying a change.

Use **Select all** to act on everything currently shown, and **Clear** to reset the selection.

> **Note:** Writing to a collection requires Steam to be closed. If Steam is running, the change is
> refused and an error banner explains why.

## 🏷️ Reading collection membership

Each row lists the collections a game already belongs to as small pills, so you can see membership
at a glance. Favorites are marked with a star, and non-Steam shortcuts carry a pill naming their
source platform.

## 📊 Columns

- **Name** — the game title, with membership pills.
- **Size** — on-disk size for installed games.

## 🧱 Creating collections automatically

For bulk work, the **Organize** view creates one collection per value of a chosen sorting option —
genre, developer, publisher, Steam feature, tag, release year, user score, **platform**, **installed
state**, **playtime**, or **name**. Choose the option, preview the result, then create the
collections. See [AutoCategorization](AutoCategorization.md).

## ➡️ Next steps

- [Collections](Collections.md) — how collections are stored and written.
- [AutoCategorization](AutoCategorization.md) — generate collections automatically.
