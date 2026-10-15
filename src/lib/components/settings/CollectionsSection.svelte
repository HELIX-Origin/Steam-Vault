<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, loading } from "$lib/stores";

  let newName = $state("");
  let busy = $state(false);

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

<div class="settings-collections-section">
  <div class="section-head-card glass-strong card" style="margin-bottom:1rem">
    <div class="row-between" style="align-items:center">
      <div>
        <h2 class="section-title" style="margin:0">{$t("collections.title")}</h2>
        <p class="muted-sm" style="margin:0.25rem 0 0">{$t("collections.subtitle")}</p>
      </div>
      <span class="pill pill-dim">{$collections.length} active</span>
    </div>
  </div>

  <div class="glass-strong toolbar" style="margin-bottom:1rem">
    <input placeholder={$t("collections.collectionName")} bind:value={newName} />
    <button class="btn btn-primary" disabled={busy || !newName.trim()} onclick={create}>{$t("collections.create")}</button>
    <span class="muted-xs">Steam must be closed to write changes.</span>
  </div>

  {#if $loading}
    <p class="muted">Loading collections…</p>
  {:else}
    <div class="glass list">
      {#each $collections as collection (collection.id)}
        <div class="row item">
          <div class="item-info">
            <strong>{collection.name}</strong>
            {#if collection.favorite}<span class="pill">Favorites</span>{/if}
            {#if collection.hidden}<span class="pill pill-dim">Hidden</span>{/if}
            {#if collection.dynamic}<span class="pill pill-accent2">Smart Collection</span>{/if}
          </div>
          <div class="item-actions">
            <span class="muted-sm">{collection.added.length} games</span>
            {#if collection.dynamic}
              <button class="btn btn-ghost" disabled={busy} onclick={() => refreshSmart(collection.id)}>
                ↻ Re-evaluate
              </button>
            {/if}
            {#if !collection.favorite && !collection.hidden}
              <button class="btn btn-danger btn-ghost" disabled={busy} onclick={() => remove(collection.id)}>Delete</button>
            {/if}
          </div>
        </div>
      {/each}
      {#if $collections.length === 0}
        <p class="muted empty-state">No collections found for this account yet.</p>
      {/if}
    </div>
  {/if}
</div>
