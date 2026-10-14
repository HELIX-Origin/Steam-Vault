<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { collections, error, games, loading } from "$lib/stores";
  import type { Game } from "$lib/types";

  let search = "";
  let collectionFilter: string | null = null;
  let onlyInstalled = false;
  let selected = new Set<number>();
  let targetCollection: string | null = null;
  let busy = false;

  onMount(async () => {
    loading.set(true);
    try {
      const [lib, cols] = await Promise.all([api.loadLibrary(), api.loadCollections()]);
      games.set(lib);
      collections.set(cols);
      targetCollection = cols[0]?.name ?? null;
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      loading.set(false);
    }
  });

  $: filtered = $games.filter((game) => {
    if (search && !game.name.toLowerCase().includes(search.toLowerCase())) return false;
    if (onlyInstalled && !game.installed) return false;
    if (collectionFilter && !game.collections.includes(collectionFilter)) return false;
    return true;
  });

  function toggleSelection(appId: number) {
    const next = new Set(selected);
    next.has(appId) ? next.delete(appId) : next.add(appId);
    selected = next;
  }

  function selectAll() {
    selected = new Set(filtered.map((game) => game.appId));
  }

  function clearSelection() {
    selected = new Set();
  }

  async function assign(mode: "add" | "remove") {
    if (!targetCollection || selected.size === 0) return;
    busy = true;
    try {
      const collection = $collections.find((c) => c.name === targetCollection);
      if (!collection) throw new Error("Collection not found");
      await api.updateCollection({
        collectionId: collection.id,
        addAppIds: mode === "add" ? [...selected] : [],
        removeAppIds: mode === "remove" ? [...selected] : []
      });
      const [lib, cols] = await Promise.all([api.loadLibrary(), api.loadCollections()]);
      games.set(lib);
      collections.set(cols);
      selected = new Set();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  function formatSize(bytes: number): string {
    if (!bytes) return "—";
    const gb = bytes / 1024 ** 3;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1024 ** 2).toFixed(0)} MB`;
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>Library</h1>
    <p class="muted">{$games.length} entries · {filtered.length} shown · {selected.size} selected</p>
  </div>
  <div class="row">
    <button class="btn" onclick={selectAll}>Select all</button>
    <button class="btn btn-ghost" onclick={clearSelection} disabled={selected.size === 0}>Clear</button>
  </div>
</div>

<div class="glass-strong toolbar">
  <input placeholder="Search games…" bind:value={search} style="max-width:280px" />
  <select bind:value={collectionFilter} style="max-width:220px">
    <option value={null}>All collections</option>
    {#each $collections as collection (collection.id)}
      <option value={collection.name}>{collection.name}</option>
    {/each}
  </select>
  <label class="row" style="margin:0;gap:0.4rem">
    <input type="checkbox" bind:checked={onlyInstalled} style="width:auto" /> Installed only
  </label>
  <div class="spacer"></div>
  <select bind:value={targetCollection} style="max-width:200px">
    {#each $collections as collection (collection.id)}
      <option value={collection.name}>{collection.name}</option>
    {/each}
  </select>
  <button class="btn btn-primary" disabled={busy || selected.size === 0} onclick={() => assign("add")}>Add</button>
  <button class="btn btn-danger" disabled={busy || selected.size === 0} onclick={() => assign("remove")}>Remove</button>
</div>

{#if $loading}
  <p class="muted">Loading library…</p>
{:else}
  <div class="glass list">
    {#each filtered as game (game.kind + "-" + game.appId)}
      <button class="row game" class:selected={selected.has(game.appId)} onclick={() => toggleSelection(game.appId)}>
        <span class="check">{selected.has(game.appId) ? "✓" : ""}</span>
        <span class="name">
          {game.name || `App ${game.appId}`}
          {#if game.kind === "shortcut"}<span class="pill pill-accent2">non-Steam</span>{/if}
          {#if game.favorite}<span class="pill">★</span>{/if}
        </span>
        <span class="tags">
          {#each game.collections as collection}
            <span class="pill pill-dim">{collection}</span>
          {/each}
        </span>
        <span class="muted-sm size">{formatSize(game.sizeOnDisk)}</span>
      </button>
    {/each}
    {#if filtered.length === 0}
      <p class="muted" style="padding:1rem">No games match the current filters.</p>
    {/if}
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.7rem 0.9rem;
    margin-bottom: 0.9rem;
    flex-wrap: wrap;
  }

  .spacer {
    flex: 1;
  }

  .list {
    padding: 0.35rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .game {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    text-align: left;
    padding: 0.55rem 0.7rem;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font: inherit;
  }

  .game:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .game.selected {
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.14), rgba(176, 108, 255, 0.12));
    border-color: var(--border-strong);
  }

  .check {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    border: 1px solid var(--border-strong);
    display: grid;
    place-items: center;
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .tags {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
    justify-content: flex-end;
    max-width: 40%;
  }

  .size {
    width: 68px;
    text-align: right;
    flex-shrink: 0;
  }
</style>
