<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { error, ownedLibrary } from "$lib/stores";
  import type {
    CategorizationPlan,
    CategorizeKind,
    CategorizeRequest,
    MetadataSyncResult
  } from "$lib/types";

  const options: { kind: CategorizeKind; label: string; hint: string }[] = [
    { kind: "genre", label: "Genre", hint: "One collection per genre, e.g. Action, RPG." },
    { kind: "developer", label: "Developer", hint: "One collection per developer." },
    { kind: "publisher", label: "Publisher", hint: "One collection per publisher." },
    { kind: "flag", label: "Steam feature", hint: "Categories like Co-op, Controller support." },
    { kind: "tag", label: "Tag", hint: "Community tags such as Roguelike." },
    { kind: "releaseyear", label: "Release year", hint: "A decade bucket plus the exact year." },
    { kind: "userscore", label: "User score", hint: "Score buckets, 90+, 80–89, …" },
    { kind: "platform", label: "Platform", hint: "Steam, GOG, Epic, itch.io, …" },
    { kind: "installedstate", label: "Installed state", hint: "Installed vs. not installed." },
    { kind: "playtime", label: "Playtime", hint: "Unplayed, <2h, 2–10h, 10–50h, 50h+." },
    { kind: "name", label: "Name (A–Z)", hint: "First letter of each game's name." }
  ];

  let kind: CategorizeKind = "genre";
  let prefix = "";
  let removeExisting = false;
  let onlyInstalled = false;
  let plan: CategorizationPlan | null = null;
  let busy = false;
  let syncingMeta = false;
  let meta: MetadataSyncResult | null = null;
  let applied: string | null = null;

  onMount(async () => {
    try {
      ownedLibrary.set(await api.getOwnedLibrary());
    } catch (e) {
      error.set(errorMessage(e));
    }
  });

  $: current = options.find((option) => option.kind === kind);
  $: optionsKey = `${kind}|${prefix}|${removeExisting}|${onlyInstalled}`;

  let lastKey = "";
  $: if (optionsKey !== lastKey) {
    lastKey = optionsKey;
    plan = null;
  }

  function request(): CategorizeRequest {
    return {
      kind,
      prefix: prefix.trim() || null,
      removeExisting,
      onlyInstalled,
      fetchMetadata: true
    };
  }

  async function syncOwned() {
    busy = true;
    try {
      ownedLibrary.set(await api.refreshOwnedGames());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function syncMetadata() {
    syncingMeta = true;
    try {
      meta = await api.syncLibraryMetadata();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      syncingMeta = false;
    }
  }

  async function preview() {
    busy = true;
    applied = null;
    try {
      plan = await api.computeCategorization(request());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function apply() {
    if (!plan) return;
    busy = true;
    try {
      const result = await api.applyCategorization(plan);
      applied = `Created ${result.collectionsCreated} new · updated ${result.collectionsUpdated} · ${result.gamesAssigned} game assignments.`;
      plan = null;
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  function totalGames(): number {
    return plan?.assignments.reduce((sum, item) => sum + item.appIds.length, 0) ?? 0;
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>Organize</h1>
    <p class="muted">
      Pick any sorting option to build Steam collections from your whole library.
    </p>
  </div>
</div>

<div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
  <div class="row-between">
    <div>
      <strong>Full owned library</strong>
      <p class="muted-sm" style="margin:0.2rem 0 0">
        {#if $ownedLibrary?.fetchedAt}
          {$ownedLibrary.games.length} owned games cached ·
          {new Date($ownedLibrary.fetchedAt * 1000).toLocaleString()}
        {:else}
          Not synced yet. Sync to organize your whole Steam library, not only installed games.
        {/if}
      </p>
    </div>
    <button class="btn" onclick={syncOwned} disabled={busy}>
      {busy ? "Working…" : "Sync owned games"}
    </button>
    <button class="btn btn-ghost" onclick={syncMetadata} disabled={syncingMeta}>
      {syncingMeta ? "Fetching metadata…" : "Sync metadata"}
    </button>
  </div>
</div>

<div class="glass-strong" style="padding:1rem 1.1rem;margin-bottom:1rem">
  <div class="section-title">Create collections from…</div>
  <div class="grid choices">
    {#each options as option (option.kind)}
      <button
        class="choice"
        class:active={kind === option.kind}
        onclick={() => (kind = option.kind)}
        title={option.hint}
      >
        <span class="choice-label">{option.label}</span>
        <span class="muted-sm">{option.hint}</span>
      </button>
    {/each}
  </div>

  <div class="row opts">
    <label class="row" style="margin:0;gap:0.4rem">
      <input type="checkbox" bind:checked={removeExisting} style="width:auto" />
      Replace existing membership
    </label>
    <label class="row" style="margin:0;gap:0.4rem">
      <input type="checkbox" bind:checked={onlyInstalled} style="width:auto" />
      Installed games only
    </label>
    <input placeholder="Optional prefix…" bind:value={prefix} style="max-width:220px" />
    <div class="spacer"></div>
    <button class="btn btn-primary" onclick={preview} disabled={busy}>
      Preview collections
    </button>
  </div>
</div>

{#if meta}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <strong>Metadata</strong> · {meta.requested} checked · {meta.fetched} fetched
    {#if meta.tagsFetched}· {meta.tagsFetched} tag sets{/if}
    {#if meta.cached}· {meta.cached} cached{/if}
    {#if meta.unavailable}· {meta.unavailable} unavailable{/if}
  </div>
{/if}

{#if plan}
  <div class="row-between" style="margin-bottom:0.4rem">
    <div>
      <h2 style="margin:0">Preview · {current?.label}</h2>
      <p class="muted-sm">
        {plan.assignments.length} collections · {plan.categorizedGames} of {plan.totalGames} games
        matched
        {#if plan.gamesWithoutMetadata.length}
          · {plan.gamesWithoutMetadata.length} without metadata
        {/if}
      </p>
    </div>
    <button class="btn btn-primary" onclick={apply} disabled={busy}>Create collections</button>
  </div>

  <div class="glass list">
    {#each plan.assignments as group (group.collection)}
      <div class="row group">
        <span class="name">{group.collection}</span>
        <span class="pill">{group.appIds.length}</span>
      </div>
    {/each}
    {#if plan.assignments.length === 0}
      <p class="muted" style="padding:1rem">
        No games matched this sorting option. Fetch store metadata or turn off the installed-only
        filter.
      </p>
    {/if}
  </div>
{/if}

<style>
  .choices {
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    margin: 0.8rem 0 1rem;
  }

  .choice {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.7em 0.85em;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.02);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .choice:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .choice.active {
    border-color: var(--border-strong);
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.16), rgba(176, 108, 255, 0.14));
  }

  .choice-label {
    font-weight: 600;
  }

  .opts {
    gap: 0.8rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .spacer {
    flex: 1;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px;
  }

  .group {
    padding: 0.5em 0.7em;
    border-radius: var(--radius-sm);
  }

  .group:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .name {
    flex: 1;
  }
</style>

