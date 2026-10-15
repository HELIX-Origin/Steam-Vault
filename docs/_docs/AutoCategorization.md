---
layout: docs
title: AutoCategorization
summary: The AutoCats engine for bulk collection building
parent: Documentation Home
nav_order: 5
---

# AutoCategorization

**AutoCats** are rules that create collections for you, in the spirit of Depressurizer. Instead of
filing games one by one, you describe a rule such as "one collection per genre" and Steam Vault
proposes the result.

## 🧠 How a rule works

Each AutoCat has:

- a **kind** — what it reads from metadata;
- an optional **prefix** added to every generated collection name (for example `Genre: `);
- an optional **maximum number of categories**;
- **include** and **exclude** lists to fine-tune matches;
- a **remove existing** option to replace, rather than merge with, prior membership.

## 🏷️ Supported kinds

| Kind | Generates a collection per… |
| :--- | :--- |
| Genre | genre reported by the metadata providers |
| Developer | developing studio |
| Publisher | publishing company |
| Flag | Steam store flag (for example, single-player) |
| Tag | community tag |
| Release year | year the game was released |
| User score | score band |
| Platform | source storefront (Steam, GOG, Epic, …) |
| Installed state | whether the game is installed locally |
| Playtime | playtime band (unplayed, `<2h`, 2–10h, 10–50h, 50h+) |
| Name | first letter of the game's name |

## 🧱 Create collections from a sorting option

The **Organize** view exposes the same kinds as a one-shot action: choose a sorting option, preview
the collections it would create, then write them all in a single, backed-up operation. Options
include a name prefix, an installed-only filter, and a replace-membership mode.

The same view includes a **filter builder** — Allowed / Required / Excluded term lists plus a name
search and a kind selector — with a live count of matching games, so you can narrow exactly which
games a sorting option acts on before you apply it.

## 🔄 Compute, review, apply

AutoCats run in two stages:

1. **Compute** builds a **plan** — the collections it would create and the games that would go in
   them — without touching Steam. Games missing metadata are reported so you can fetch it first.
2. **Apply** writes the plan into your collections. As always, Steam must be closed and a backup is
   taken first.

> **Note:** Because AutoCats depend on metadata, run a metadata fetch (see
> [Metadata Providers](Metadata-Providers.md)) before computing a plan to get complete results.

## 🎛️ Beyond AutoCats

For arbitrary, ad-hoc groupings, use the **Allowed / Required / Excluded** filter model: a game must
match at least one *Allowed* term (OR), all *Required* terms (AND), and no *Excluded* term (NOT).
This reproduces Depressurizer's advanced filtering without writing a rule.

## ➡️ Next steps

- [Collections](Collections.md)
- [Metadata Providers](Metadata-Providers.md)
