<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, loading } from "$lib/stores";
  import type { Game, Platform } from "$lib/types";

  let search = "";
  let collectionFilter: string | null = null;
  let onlyInstalled = false;
  let selected = new Set<number>();
  let targetCollection: string | null = null;
  let busy = false;
  let syncing = false;
  let bulkPlatform = "";

  const platformLabels: Record<string, string> = {
    steam: "Steam",
    gog: "GOG",
    epic: "Epic",
    itch: "itch.io",
    ubisoft: "Ubisoft",
    ea: "EA",
    battlenet: "Battle.net",
    amazon: "Amazon",
    xbox: "Xbox",
    rockstar: "Rockstar",
    emulator: "Emulator",
    other: "Other"
  };

  const platformOptions: { id: Platform; label: string }[] = [
    { id: "steam", label: "Steam" },
    { id: "gog", label: "GOG" },
    { id: "epic", label: "Epic" },
    { id: "itch", label: "itch.io" },
    { id: "ubisoft", label: "Ubisoft" },
    { id: "ea", label: "EA" },
    { id: "battlenet", label: "Battle.net" },
    { id: "amazon", label: "Amazon" },
    { id: "xbox", label: "Xbox" },
    { id: "rockstar", label: "Rockstar" },
    { id: "emulator", label: "Emulator" },
    { id: "other", label: "Other" }
  ];

  $: selectedShortcuts = $games.filter((game) => selected.has(game.appId) && game.kind === "shortcut");

  async function setPlatform(game: Game, platform: string) {
    try {
      await api.setGamePlatform(game.appId, platform);
      game.platform = platform as Platform;
      games.set([...$games]);
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  async function applyBulkPlatform() {
    if (!bulkPlatform || selectedShortcuts.length === 0) return;
    busy = true;
    try {
      for (const s of selectedShortcuts) {
        await api.setGamePlatform(s.appId, bulkPlatform);
        s.platform = bulkPlatform as Platform;
      }
      games.set([...$games]);
      bulkPlatform = "";
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  $: installedCount = $games.filter((game) => game.installed).length;

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

  async function syncOwned() {
    syncing = true;
    try {
      await api.refreshOwnedGames();
      games.set(await api.loadLibrary());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      syncing = false;
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
    <h1>{$t("nav.library")}</h1>
    <p class="muted">
      {$games.length} owned · {installedCount} installed · {filtered.length} shown · {selected.size} selected
    </p>
  </div>
  <div class="row">
    <button class="btn" onclick={syncOwned} disabled={syncing}>
      {syncing ? $t("common.refreshing") : "Sync owned games"}
    </button>
    <button class="btn" onclick={selectAll}>{$t("common.all")}</button>
    <button class="btn btn-ghost" onclick={clearSelection} disabled={selected.size === 0}>{$t("common.clear")}</button>
  </div>
</div>

<div class="glass-strong toolbar">
  <input placeholder={$t("library.searchPlaceholder")} bind:value={search} style="max-width:280px" />
  <select bind:value={collectionFilter} style="max-width:220px">
    <option value={null}>{$t("library.allCategories")}</option>
    {#each $collections as collection (collection.id)}
      <option value={collection.name}>{collection.name}</option>
    {/each}
  </select>
  <label class="row" style="margin:0;gap:0.4rem">
    <input type="checkbox" bind:checked={onlyInstalled} style="width:auto" /> {$t("library.installedOnly")}
  </label>
  <div class="spacer"></div>
  {#if selectedShortcuts.length > 0}
    <select bind:value={bulkPlatform} style="max-width:140px">
      <option value="">Set platform…</option>
      {#each platformOptions as opt}
        <option value={opt.id}>{opt.label}</option>
      {/each}
    </select>
    <button class="btn" disabled={busy || !bulkPlatform} onclick={applyBulkPlatform}>Tag</button>
  {/if}
  <select bind:value={targetCollection} style="max-width:200px">
    {#each $collections as collection (collection.id)}
      <option value={collection.name}>{collection.name}</option>
    {/each}
  </select>
  <button class="btn btn-primary" disabled={busy || selected.size === 0} onclick={() => assign("add")}>{$t("common.add")}</button>
  <button class="btn btn-danger" disabled={busy || selected.size === 0} onclick={() => assign("remove")}>{$t("common.remove")}</button>
</div>

{#if $loading}
  <p class="muted">{$t("common.loading")}</p>
{:else}
  <div class="glass list">
    {#each filtered as game (game.kind + "-" + game.appId)}
      <div
        class="row game"
        class:selected={selected.has(game.appId)}
        role="button"
        tabindex="0"
        onclick={() => toggleSelection(game.appId)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            toggleSelection(game.appId);
          }
        }}
      >
        <span class="check">{selected.has(game.appId) ? "✓" : ""}</span>
        <span class="name">
          {game.name || `App ${game.appId}`}
          {#if game.kind === "shortcut"}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="platform-tag" onclick={(e) => e.stopPropagation()}>
              <select
                class="platform-select pill pill-accent2"
                value={game.platform}
                onchange={(e) => setPlatform(game, e.currentTarget.value)}
                title="Change source platform"
              >
                {#each platformOptions as opt}
                  <option value={opt.id}>{opt.label}</option>
                {/each}
              </select>
            </span>
          {/if}
          {#if !game.installed}<span class="pill pill-dim">not installed</span>{/if}
          {#if game.favorite}<span class="pill">★</span>{/if}
        </span>
        <span class="tags">
          {#each game.collections as collection}
            <span class="pill pill-dim">{collection}</span>
          {/each}
        </span>
        <span class="muted-sm size">{formatSize(game.sizeOnDisk)}</span>
      </div>
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

  .platform-tag {
    display: inline-flex;
    align-items: center;
  }

  .platform-select {
    cursor: pointer;
    border: none;
    background: transparent;
    color: inherit;
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.15rem 0.45rem;
    outline: none;
    border-radius: var(--radius-sm, 6px);
  }

  .platform-select option {
    background: var(--bg-card, #1c1d22);
    color: var(--fg, #e6e6e6);
  }
</style>
