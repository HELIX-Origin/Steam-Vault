<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, loading } from "$lib/stores";

  let newName = "";
  let busy = false;

  async function reload() {
    const [lib, cols] = await Promise.all([api.loadLibrary(), api.loadCollections()]);
    games.set(lib);
    collections.set(cols);
  }

  onMount(async () => {
    loading.set(true);
    try {
      await reload();
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      loading.set(false);
    }
  });

  async function create() {
    if (!newName.trim()) return;
    busy = true;
    try {
      await api.createCollection(newName.trim());
      newName = "";
      await reload();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function refreshSmart(id: string) {
    busy = true;
    try {
      await api.refreshSmartCollection(id);
      await reload();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function remove(id: string) {
    busy = true;
    try {
      await api.deleteCollection(id);
      await reload();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div style="margin-bottom:1rem">
  <h1>{$t("collections.title")}</h1>
  <p class="muted">{$t("collections.subtitle")}</p>
</div>

<div class="glass-strong toolbar">
  <input placeholder={$t("collections.collectionName")} bind:value={newName} style="max-width:300px" />
  <button class="btn btn-primary" disabled={busy || !newName.trim()} onclick={create}>{$t("collections.create")}</button>
  <span class="muted-sm">Steam must be closed to write changes.</span>
</div>

{#if $loading}
  <p class="muted">Loading collections…</p>
{:else}
  <div class="glass list">
    {#each $collections as collection (collection.id)}
      <div class="row item">
        <div class="row" style="gap:0.5rem;align-items:center">
          <strong>{collection.name}</strong>
          {#if collection.favorite}<span class="pill">Favorites</span>{/if}
          {#if collection.hidden}<span class="pill pill-dim">Hidden</span>{/if}
          {#if collection.dynamic}<span class="pill pill-accent2">Smart Collection</span>{/if}
        </div>
        <div class="row" style="gap:0.5rem;align-items:center">
          <span class="muted-sm">{collection.added.length} games</span>
          {#if collection.dynamic}
            <button class="btn btn-ghost" style="padding:0.25rem 0.6rem;font-size:0.8rem" disabled={busy} onclick={() => refreshSmart(collection.id)}>
              ↻ Re-evaluate
            </button>
          {/if}
          {#if !collection.favorite && !collection.hidden}
            <button class="btn btn-danger btn-ghost" style="padding:0.25rem 0.6rem;font-size:0.8rem" disabled={busy} onclick={() => remove(collection.id)}>Delete</button>
          {/if}
        </div>
      </div>
    {/each}
    {#if $collections.length === 0}
      <p class="muted" style="padding:1rem">No collections found for this account yet.</p>
    {/if}
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .list {
    padding: 0.45rem;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .item {
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: center;
    gap: 1rem;
    padding: 0.65rem 0.85rem;
    border-radius: var(--radius-sm);
    border-bottom: 1px solid var(--border-subtle);
    transition: background 0.15s ease;
  }

  .item:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .item:last-child {
    border-bottom: none;
  }
</style>
