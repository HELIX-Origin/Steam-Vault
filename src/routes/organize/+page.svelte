<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, ownedLibrary } from "$lib/stores";
  import type {
    CategorizationPlan,
    CategorizeKind,
    CategorizeRequest,
    Filter,
    FilterPreset,
    Game,
    GameKind,
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

  // Filter builder (Depressurizer-style Allowed / Required / Excluded).
  let filterAllowed: string[] = [];
  let filterRequired: string[] = [];
  let filterExcluded: string[] = [];
  let filterSearch = "";
  let filterKind: GameKind | "" = "";

  // Named presets and Smart Collections.
  let presets: FilterPreset[] = [];
  let presetName = "";
  let selectedPresetId = "";
  let smartCollectionName = "";

  $: collectionNames = $collections
    .map((collection) => collection.name)
    .sort((a, b) => a.localeCompare(b));
  $: gameCount = $games.length;
  $: matchCount = $games.filter((game) => matchesFilter(game)).length;

  function hasCollection(game: Game, name: string): boolean {
    return game.collections.some((entry) => entry.toLowerCase() === name.toLowerCase());
  }

  /** Mirror of the backend `filter::matches` so the count updates live. */
  function matchesFilter(game: Game): boolean {
    if (filterKind && game.kind !== filterKind) return false;
    const search = filterSearch.trim().toLowerCase();
    if (search && !game.name.toLowerCase().includes(search)) return false;
    if (filterAllowed.length && !filterAllowed.some((name) => hasCollection(game, name))) {
      return false;
    }
    if (filterRequired.length && !filterRequired.every((name) => hasCollection(game, name))) {
      return false;
    }
    if (filterExcluded.length && filterExcluded.some((name) => hasCollection(game, name))) {
      return false;
    }
    return true;
  }

  function currentFilter(): Filter | null {
    const search = filterSearch.trim();
    if (
      !filterAllowed.length &&
      !filterRequired.length &&
      !filterExcluded.length &&
      !search &&
      !filterKind
    ) {
      return null;
    }
    return {
      allowed: filterAllowed,
      required: filterRequired,
      excluded: filterExcluded,
      search: search || null,
      kind: filterKind || null
    };
  }

  function clearFilter() {
    filterAllowed = [];
    filterRequired = [];
    filterExcluded = [];
    filterSearch = "";
    filterKind = "";
    selectedPresetId = "";
  }

  async function loadPresets() {
    try {
      presets = await api.getFilterPresets();
    } catch {
      // ignore
    }
  }

  function applyPreset(id: string) {
    const p = presets.find((item) => item.id === id);
    if (!p) return;
    filterAllowed = p.filter.allowed ? [...p.filter.allowed] : [];
    filterRequired = p.filter.required ? [...p.filter.required] : [];
    filterExcluded = p.filter.excluded ? [...p.filter.excluded] : [];
    filterSearch = p.filter.search ?? "";
    filterKind = p.filter.kind ?? "";
  }

  async function saveCurrentPreset() {
    const f = currentFilter();
    if (!presetName.trim() || !f) return;
    busy = true;
    try {
      const p = await api.saveFilterPreset({
        id: selectedPresetId || `preset-${Date.now()}`,
        name: presetName.trim(),
        filter: f,
        createdAt: Date.now()
      });
      presetName = "";
      await loadPresets();
      selectedPresetId = p.id;
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function deleteSelectedPreset() {
    if (!selectedPresetId) return;
    busy = true;
    try {
      await api.deleteFilterPreset(selectedPresetId);
      selectedPresetId = "";
      await loadPresets();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function saveAsSmartCollection() {
    const f = currentFilter();
    if (!smartCollectionName.trim() || !f) return;
    busy = true;
    try {
      await api.saveSmartCollection(smartCollectionName.trim(), f);
      applied = `Saved smart collection "${smartCollectionName.trim()}" with ${matchCount} matching games.`;
      smartCollectionName = "";
      collections.set(await api.loadCollections());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  onMount(async () => {
    try {
      const [owned, library, savedCollections] = await Promise.all([
        api.getOwnedLibrary(),
        api.loadLibrary(),
        api.loadCollections()
      ]);
      ownedLibrary.set(owned);
      games.set(library);
      collections.set(savedCollections);
      await loadPresets();
    } catch (e) {
      error.set(errorMessage(e));
    }
  });

  $: current = options.find((option) => option.kind === kind);
  $: optionsKey = `${kind}|${prefix}|${removeExisting}|${onlyInstalled}|${JSON.stringify(currentFilter())}`;

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
      fetchMetadata: true,
      filter: currentFilter()
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
    <h1>{$t("organize.title")}</h1>
    <p class="muted">
      {$t("organize.subtitle")}
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
  <div class="row-between">
    <div class="section-title">Filter (optional)</div>
    <span class="muted-sm">{matchCount} of {gameCount} games match</span>
  </div>
  <p class="muted-sm" style="margin:0 0 0.7rem">
    Narrow which games are organized before collections are created. Empty fields are ignored.
    Hold Ctrl (⌘) to pick several per list.
  </p>
  <div class="filter-grid">
    <label class="field">
      <span>Allowed (any of)</span>
      <select multiple bind:value={filterAllowed}>
        {#each collectionNames as name (name)}<option value={name}>{name}</option>{/each}
      </select>
    </label>
    <label class="field">
      <span>Required (all of)</span>
      <select multiple bind:value={filterRequired}>
        {#each collectionNames as name (name)}<option value={name}>{name}</option>{/each}
      </select>
    </label>
    <label class="field">
      <span>Excluded (none of)</span>
      <select multiple bind:value={filterExcluded}>
        {#each collectionNames as name (name)}<option value={name}>{name}</option>{/each}
      </select>
    </label>
    <label class="field">
      <span>Name contains</span>
      <input bind:value={filterSearch} placeholder="e.g. doom" />
    </label>
    <label class="field">
      <span>Kind</span>
      <select bind:value={filterKind}>
        <option value="">Any</option>
        <option value="steam">Steam</option>
        <option value="shortcut">Non-Steam shortcut</option>
      </select>
    </label>
  </div>
  <div class="row" style="gap:0.7rem;margin-top:0.9rem;align-items:center;flex-wrap:wrap">
    <button class="btn btn-ghost" onclick={clearFilter}>Clear filter</button>
    {#if presets.length > 0}
      <select bind:value={selectedPresetId} onchange={() => applyPreset(selectedPresetId)} style="max-width:200px">
        <option value="">Load saved preset…</option>
        {#each presets as p (p.id)}
          <option value={p.id}>{p.name}</option>
        {/each}
      </select>
      {#if selectedPresetId}
        <button class="btn btn-ghost btn-danger" style="font-size:0.8rem;padding:0.25rem 0.5rem" onclick={deleteSelectedPreset} disabled={busy}>Delete</button>
      {/if}
    {/if}
    <input placeholder="New preset name…" bind:value={presetName} style="max-width:180px" />
    <button class="btn btn-ghost" onclick={saveCurrentPreset} disabled={busy || !presetName.trim() || !currentFilter()}>
      Save preset
    </button>
    {#if collectionNames.length === 0}
      <span class="muted-sm">No collections to filter by yet — create some first.</span>
    {/if}
  </div>

  <div class="row" style="gap:0.7rem;margin-top:0.8rem;padding-top:0.8rem;border-top:1px solid rgba(255,255,255,0.08);align-items:center;flex-wrap:wrap">
    <span class="muted-sm" style="font-weight:600">Smart Collection:</span>
    <input placeholder="Smart collection name…" bind:value={smartCollectionName} style="max-width:220px" />
    <button class="btn btn-accent" onclick={saveAsSmartCollection} disabled={busy || !smartCollectionName.trim() || !currentFilter()}>
      Save as Smart Collection
    </button>
    <span class="muted-sm">Creates a dynamic Steam collection that can be re-evaluated anytime.</span>
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
    margin: 0.8rem 0 1.1rem;
    gap: 0.75rem;
  }

  .filter-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 0.8rem;
  }

  .filter-grid select[multiple] {
    min-height: 100px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field span {
    font-size: 0.78rem;
    color: var(--text-mute);
  }

  .choice {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.75em 0.9em;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.02);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
    box-shadow: var(--glass-highlight);
    transition: border-color 0.16s ease, background 0.16s ease, transform 0.12s ease, box-shadow 0.16s ease;
  }

  .choice:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }

  .choice.active {
    border-color: var(--border-glow);
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.2), rgba(168, 85, 247, 0.16));
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.14), 0 4px 16px rgba(56, 189, 248, 0.15);
  }

  .choice-label {
    font-weight: 600;
    font-size: 0.92rem;
  }

  .opts {
    gap: 0.85rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .spacer {
    flex: 1;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 8px;
  }

  .group {
    padding: 0.55em 0.75em;
    border-radius: var(--radius-sm);
    transition: background 0.14s ease;
  }

  .group:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .name {
    flex: 1;
    font-weight: 550;
  }
</style>

