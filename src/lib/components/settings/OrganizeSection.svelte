<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, ownedLibrary } from "$lib/stores";
  import type {
    CategorizationPlan,
    CategorizeKind,
    Filter,
    FilterPreset,
    Game,
    GameKind,
    MetadataSyncResult
  } from "$lib/types";

  // Built-in organization scope definition
  interface BuiltinScope {
    kind: CategorizeKind;
    label: string;
    hint: string;
    icon: string;
  }

  const builtinScopes: BuiltinScope[] = [
    { kind: "genre", label: "Genre", hint: "One collection per genre (e.g. Action, RPG, Strategy).", icon: "🏷️" },
    { kind: "developer", label: "Developer", hint: "One collection per development studio.", icon: "💻" },
    { kind: "publisher", label: "Publisher", hint: "One collection per game publisher.", icon: "🏢" },
    { kind: "flag", label: "Steam Feature Flag", hint: "Feature categories like Co-op, Full Controller, VR Support.", icon: "🎮" },
    { kind: "tag", label: "Community Tags", hint: "Popular Steam community tags (Roguelike, Open World, etc.).", icon: "🔖" },
    { kind: "releaseyear", label: "Release Year & Decade", hint: "Decade collections plus exact release year buckets.", icon: "📅" },
    { kind: "userscore", label: "Review & User Score", hint: "Score tiers (90% Overwhelmingly Positive, 80-89%, etc.).", icon: "⭐" },
    { kind: "platform", label: "Storefront Platform", hint: "Collections by origin (Steam, GOG, Epic, itch.io, etc.).", icon: "🌐" },
    { kind: "installedstate", label: "Installed State", hint: "Installed games vs. Uninstalled/Cloud library.", icon: "💾" },
    { kind: "playtime", label: "Playtime Tiers", hint: "Unplayed, <2 hours, 2-10 hours, 10-50 hours, 50+ hours.", icon: "⏱️" },
    { kind: "name", label: "Alphabetical (A–Z)", hint: "First letter of each game's title for quick indexing.", icon: "🔤" }
  ];

  // Depressurizer-style custom rule scope
  interface CustomScope {
    id: string;
    name: string;
    field: "name" | "genre" | "developer" | "publisher" | "flag" | "tag" | "platform" | "playtime";
    operator: "contains" | "equals" | "not_contains" | "less_than" | "greater_than";
    value: string;
    targetCollection: string;
    enabled: boolean;
  }

  // Selected built-in scopes
  let selectedScopes = $state<Set<CategorizeKind>>(new Set(["genre"]));

  // Custom scopes list
  let customScopes = $state<CustomScope[]>([
    {
      id: "scope-unplayed",
      name: "Unplayed Backlog",
      field: "playtime",
      operator: "equals",
      value: "0",
      targetCollection: "Backlog: Unplayed",
      enabled: false
    },
    {
      id: "scope-coop",
      name: "Co-Op Multiplayer",
      field: "flag",
      operator: "contains",
      value: "Co-op",
      targetCollection: "Multiplayer Co-Op",
      enabled: false
    }
  ]);

  // Custom scope builder modal / form state
  let showCustomScopeModal = $state(false);
  let newScopeName = $state("");
  let newScopeField = $state<CustomScope["field"]>("genre");
  let newScopeOperator = $state<CustomScope["operator"]>("contains");
  let newScopeValue = $state("");
  let newScopeTarget = $state("");

  // Organization options
  let prefix = $state("");
  let removeExisting = $state(false);
  let onlyInstalled = $state(false);
  let plan = $state<CategorizationPlan | null>(null);
  let busy = $state(false);
  let syncingMeta = $state(false);
  let meta = $state<MetadataSyncResult | null>(null);
  let applied = $state<string | null>(null);

  // Filter builder (Allowed / Required / Excluded)
  let filterAllowed = $state<string[]>([]);
  let filterRequired = $state<string[]>([]);
  let filterExcluded = $state<string[]>([]);
  let filterSearch = $state("");
  let filterKind = $state<GameKind | "">("");

  // Named presets & Smart Collections
  let presets = $state<FilterPreset[]>([]);
  let presetName = $state("");
  let selectedPresetId = $state("");
  let smartCollectionName = $state("");

  let collectionNames = $derived(
    $collections
      .map((c) => c.name)
      .sort((a, b) => a.localeCompare(b))
  );
  let gameCount = $derived($games.length);
  let matchCount = $derived($games.filter((game) => matchesFilter(game)).length);

  function hasCollection(game: Game, name: string): boolean {
    return game.collections.some((entry) => entry.toLowerCase() === name.toLowerCase());
  }

  function matchesFilter(game: Game): boolean {
    if (filterKind && game.kind !== filterKind) return false;
    const s = filterSearch.trim().toLowerCase();
    if (s && !game.name.toLowerCase().includes(s)) return false;
    if (filterAllowed.length && !filterAllowed.some((name) => hasCollection(game, name))) return false;
    if (filterRequired.length && !filterRequired.every((name) => hasCollection(game, name))) return false;
    if (filterExcluded.length && filterExcluded.some((name) => hasCollection(game, name))) return false;
    return true;
  }

  function currentFilter(): Filter | null {
    const s = filterSearch.trim();
    if (!filterAllowed.length && !filterRequired.length && !filterExcluded.length && !s && !filterKind) {
      return null;
    }
    return {
      allowed: filterAllowed,
      required: filterRequired,
      excluded: filterExcluded,
      search: s || null,
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

      const saved = localStorage.getItem("steam_vault_custom_scopes");
      if (saved) {
        try {
          customScopes = JSON.parse(saved);
        } catch {
          // ignore
        }
      }
    } catch (e) {
      error.set(errorMessage(e));
    }
  });

  function saveCustomScopes() {
    try {
      localStorage.setItem("steam_vault_custom_scopes", JSON.stringify(customScopes));
    } catch {
      // ignore
    }
  }

  function toggleScope(k: CategorizeKind) {
    const next = new Set(selectedScopes);
    next.has(k) ? next.delete(k) : next.add(k);
    selectedScopes = next;
    plan = null;
  }

  function selectAllScopes() {
    selectedScopes = new Set(builtinScopes.map((b) => b.kind));
    plan = null;
  }

  function clearAllScopes() {
    selectedScopes = new Set();
    plan = null;
  }

  function addCustomScope() {
    if (!newScopeName.trim() || !newScopeTarget.trim()) return;
    customScopes = [
      ...customScopes,
      {
        id: `custom-${Date.now()}`,
        name: newScopeName.trim(),
        field: newScopeField,
        operator: newScopeOperator,
        value: newScopeValue.trim(),
        targetCollection: newScopeTarget.trim(),
        enabled: true
      }
    ];
    saveCustomScopes();
    newScopeName = "";
    newScopeValue = "";
    newScopeTarget = "";
    showCustomScopeModal = false;
    plan = null;
  }

  function removeCustomScope(id: string) {
    customScopes = customScopes.filter((s) => s.id !== id);
    saveCustomScopes();
    plan = null;
  }

  function toggleCustomScope(id: string) {
    customScopes = customScopes.map((s) => (s.id === id ? { ...s, enabled: !s.enabled } : s));
    saveCustomScopes();
    plan = null;
  }

  async function previewCombined() {
    if (selectedScopes.size === 0 && !customScopes.some((s) => s.enabled)) return;
    busy = true;
    applied = null;
    try {
      const subPlans: CategorizationPlan[] = [];
      for (const scopeKind of selectedScopes) {
        const subPlan = await api.computeCategorization({
          kind: scopeKind,
          prefix: prefix.trim() || null,
          removeExisting,
          onlyInstalled,
          fetchMetadata: true,
          filter: currentFilter()
        });
        subPlans.push(subPlan);
      }

      const activeCustom = customScopes.filter((s) => s.enabled);
      const customAssignments: Map<string, Set<number>> = new Map();

      if (activeCustom.length > 0) {
        for (const game of $games) {
          if (!matchesFilter(game)) continue;
          if (onlyInstalled && !game.installed) continue;

          for (const rule of activeCustom) {
            let matched = false;
            const targetVal = rule.value.toLowerCase();

            switch (rule.field) {
              case "name": {
                const name = game.name.toLowerCase();
                matched =
                  rule.operator === "contains"
                    ? name.includes(targetVal)
                    : rule.operator === "equals"
                    ? name === targetVal
                    : !name.includes(targetVal);
                break;
              }
              case "platform": {
                const p = game.platform.toLowerCase();
                matched =
                  rule.operator === "contains"
                    ? p.includes(targetVal)
                    : rule.operator === "equals"
                    ? p === targetVal
                    : !p.includes(targetVal);
                break;
              }
              case "playtime": {
                const hours = (game.playtimeMinutes ?? 0) / 60;
                const num = parseFloat(rule.value) || 0;
                if (rule.operator === "equals") matched = (game.playtimeMinutes ?? 0) === 0 && num === 0;
                else if (rule.operator === "less_than") matched = hours < num;
                else if (rule.operator === "greater_than") matched = hours > num;
                break;
              }
              case "genre":
              case "tag":
              case "flag":
              case "developer":
              case "publisher": {
                matched = game.collections.some((c) => c.toLowerCase().includes(targetVal));
                break;
              }
            }

            if (matched) {
              const colName = (prefix.trim() ? `${prefix.trim()} ` : "") + rule.targetCollection;
              if (!customAssignments.has(colName)) {
                customAssignments.set(colName, new Set());
              }
              customAssignments.get(colName)!.add(game.appId);
            }
          }
        }
      }

      const combinedMap = new Map<string, Set<number>>();
      for (const sp of subPlans) {
        for (const assign of sp.assignments) {
          if (!combinedMap.has(assign.collection)) {
            combinedMap.set(assign.collection, new Set());
          }
          for (const id of assign.appIds) {
            combinedMap.get(assign.collection)!.add(id);
          }
        }
      }

      for (const [colName, ids] of customAssignments.entries()) {
        if (!combinedMap.has(colName)) {
          combinedMap.set(colName, new Set());
        }
        for (const id of ids) {
          combinedMap.get(colName)!.add(id);
        }
      }

      const allAssignedIds = new Set<number>();
      for (const ids of combinedMap.values()) {
        for (const id of ids) {
          allAssignedIds.add(id);
        }
      }

      const mergedAssignments = Array.from(combinedMap.entries())
        .map(([collection, ids]) => ({
          collection,
          appIds: Array.from(ids)
        }))
        .sort((a, b) => a.collection.localeCompare(b.collection));

      plan = {
        kind: Array.from(selectedScopes)[0] || "genre",
        assignments: mergedAssignments,
        categorizedGames: allAssignedIds.size,
        totalGames: $games.length,
        gamesWithoutMetadata: subPlans[0]?.gamesWithoutMetadata ?? [],
        removeExisting
      };

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
      applied = `Created ${result.collectionsCreated} new collections · updated ${result.collectionsUpdated} · applied ${result.gamesAssigned} game assignments to Steam!`;
      plan = null;
      collections.set(await api.loadCollections());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function syncOwned() {
    busy = true;
    try {
      ownedLibrary.set(await api.refreshOwnedGames());
      games.set(await api.loadLibrary());
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
</script>

<div class="settings-organize-section">
  <div class="glass-strong card" style="margin-bottom:1.25rem">
    <div class="row-between" style="align-items:center">
      <div>
        <h2 class="section-title" style="margin:0">{$t("organize.title")} &amp; Scopes</h2>
        <p class="muted-sm" style="margin:0.25rem 0 0">
          Select multiple organization scopes or define custom Depressurizer rules to categorize your Steam library.
        </p>
      </div>
      <div class="row" style="gap:0.5rem">
        <button class="btn mini" onclick={syncOwned} disabled={busy}>
          {busy ? "Working…" : "Sync owned games"}
        </button>
        <button class="btn mini btn-ghost" onclick={syncMetadata} disabled={syncingMeta}>
          {syncingMeta ? "Fetching metadata…" : "Sync metadata"}
        </button>
      </div>
    </div>
  </div>

  {#if applied}
    <div class="glass-strong card applied-banner" style="margin-bottom:1rem">
      <span class="pill pill-ok">Success</span> {applied}
    </div>
  {/if}

  <!-- Filter criteria (optional) -->
  <div class="glass-strong card" style="margin-bottom:1.25rem">
    <div class="row-between section-head">
      <span class="section-title">Pre-filter Games (Optional)</span>
      <span class="muted-sm">{matchCount} of {gameCount} games match filter</span>
    </div>
    <p class="muted-sm" style="margin:0 0 0.75rem">
      Narrow which games will be organized. Hold Ctrl/Cmd to select multiple tags.
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
        <input bind:value={filterSearch} placeholder="e.g. Doom, Final Fantasy…" />
      </label>
      <label class="field">
        <span>Kind</span>
        <select bind:value={filterKind}>
          <option value="">Any</option>
          <option value="steam">Steam native apps</option>
          <option value="shortcut">Non-Steam shortcuts</option>
        </select>
      </label>
    </div>

    <div class="row" style="gap:0.6rem;margin-top:0.9rem;align-items:center;flex-wrap:wrap">
      <button class="btn btn-ghost mini" onclick={clearFilter}>Clear filter</button>
      {#if presets.length > 0}
        <select bind:value={selectedPresetId} onchange={() => applyPreset(selectedPresetId)} style="max-width:200px">
          <option value="">Load saved preset…</option>
          {#each presets as p (p.id)}
            <option value={p.id}>{p.name}</option>
          {/each}
        </select>
        {#if selectedPresetId}
          <button class="btn mini btn-danger" onclick={deleteSelectedPreset} disabled={busy}>✕</button>
        {/if}
      {/if}
      <input placeholder="New preset name…" bind:value={presetName} style="max-width:180px" />
      <button class="btn mini" onclick={saveCurrentPreset} disabled={busy || !presetName.trim() || !currentFilter()}>
        Save preset
      </button>
      <div class="spacer"></div>
      <span class="muted-sm" style="font-weight:600">Smart Collection:</span>
      <input placeholder="Collection name…" bind:value={smartCollectionName} style="max-width:180px" />
      <button class="btn mini btn-accent" onclick={saveAsSmartCollection} disabled={busy || !smartCollectionName.trim() || !currentFilter()}>
        Save Dynamic Collection
      </button>
    </div>
  </div>

  <!-- ORGANIZATION SCOPES (LIST VIEW - MULTI-SELECTION) -->
  <div class="glass-strong card" style="margin-bottom:1.25rem">
    <div class="row-between section-head">
      <div>
        <h2 class="section-title">Organization Scopes (Select Multiple)</h2>
        <p class="muted-sm">
          Select all scopes to generate collections from simultaneously.
        </p>
      </div>
      <div class="row" style="gap:0.4rem">
        <button class="btn mini" onclick={selectAllScopes}>Select All</button>
        <button class="btn mini btn-ghost" onclick={clearAllScopes}>Clear All</button>
        <button class="btn mini btn-primary" onclick={() => (showCustomScopeModal = true)}>
          ＋ Add Custom Depressurizer Scope
        </button>
      </div>
    </div>

    <!-- Builtin Scopes List -->
    <div class="scopes-list glass">
      {#each builtinScopes as scope (scope.kind)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="scope-row"
          class:active={selectedScopes.has(scope.kind)}
          onclick={() => toggleScope(scope.kind)}
        >
          <input
            type="checkbox"
            checked={selectedScopes.has(scope.kind)}
            onclick={(e) => e.stopPropagation()}
            onchange={() => toggleScope(scope.kind)}
            class="scope-check"
          />
          <span class="scope-icon">{scope.icon}</span>
          <div class="scope-text">
            <span class="scope-title">{scope.label}</span>
            <span class="muted-sm scope-hint">{scope.hint}</span>
          </div>
          <span class="pill pill-dim">Built-in</span>
        </div>
      {/each}
    </div>

    <!-- Custom Scopes Section -->
    {#if customScopes.length > 0}
      <div style="margin-top:1.25rem">
        <div class="row-between" style="margin-bottom:0.5rem">
          <span class="group-label">Custom Depressurizer Scopes ({customScopes.length})</span>
        </div>
        <div class="scopes-list glass">
          {#each customScopes as custom (custom.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="scope-row custom-scope-row"
              class:active={custom.enabled}
              onclick={() => toggleCustomScope(custom.id)}
            >
              <input
                type="checkbox"
                checked={custom.enabled}
                onclick={(e) => e.stopPropagation()}
                onchange={() => toggleCustomScope(custom.id)}
                class="scope-check"
              />
              <span class="scope-icon">🎯</span>
              <div class="scope-text">
                <span class="scope-title">{custom.name}</span>
                <span class="muted-sm scope-hint">
                  If <strong>{custom.field}</strong> {custom.operator.replace('_', ' ')} <em>"{custom.value}"</em> → Collection: <strong>{custom.targetCollection}</strong>
                </span>
              </div>
              <span class="pill pill-accent2">Custom</span>
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <button
                class="btn mini btn-ghost btn-danger"
                onclick={(e) => {
                  e.stopPropagation();
                  removeCustomScope(custom.id);
                }}
                title="Delete custom scope"
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Options toolbar -->
    <div class="row opts-bar" style="margin-top:1.15rem;gap:1rem;align-items:center;flex-wrap:wrap">
      <label class="row" style="margin:0;gap:0.4rem;cursor:pointer">
        <input type="checkbox" bind:checked={removeExisting} style="width:auto" />
        Replace existing collection membership
      </label>
      <label class="row" style="margin:0;gap:0.4rem;cursor:pointer">
        <input type="checkbox" bind:checked={onlyInstalled} style="width:auto" />
        Installed games only
      </label>
      <input placeholder="Optional collection prefix (e.g. [Genre])…" bind:value={prefix} style="max-width:240px" />
      <div class="spacer"></div>
      <button
        class="btn btn-primary preview-btn"
        onclick={previewCombined}
        disabled={busy || (selectedScopes.size === 0 && !customScopes.some((s) => s.enabled))}
      >
        Preview Collections ({selectedScopes.size + customScopes.filter((s) => s.enabled).length} scopes)
      </button>
    </div>
  </div>

  <!-- PREVIEW & APPLICATION PLAN -->
  {#if plan}
    <div class="glass-strong card" style="margin-bottom:2rem">
      <div class="row-between" style="margin-bottom:0.85rem">
        <div>
          <h2 class="section-title">Organization Plan Preview</h2>
          <p class="muted-sm">
            <strong>{plan.assignments.length}</strong> collections proposed · <strong>{plan.categorizedGames}</strong> of {plan.totalGames} games assigned
            {#if plan.gamesWithoutMetadata.length}
              · <span class="pill pill-danger">{plan.gamesWithoutMetadata.length} without metadata</span>
            {/if}
          </p>
        </div>
        <button class="btn btn-primary apply-btn" onclick={apply} disabled={busy}>
          🚀 Apply Collections to Steam
        </button>
      </div>

      <div class="glass plan-assignments-list">
        {#each plan.assignments as group (group.collection)}
          <div class="row-between plan-row">
            <div class="row" style="gap:0.6rem">
              <span class="collection-mark">📁</span>
              <span class="collection-name">{group.collection}</span>
            </div>
            <span class="pill pill-accent">{group.appIds.length} games</span>
          </div>
        {/each}
        {#if plan.assignments.length === 0}
          <p class="muted" style="padding:1.5rem;text-align:center">
            No games matched the selected scopes with current filters. Try selecting more scopes or clearing the filters.
          </p>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Custom Scope Builder Modal -->
  {#if showCustomScopeModal}
    <div
      class="modal-backdrop"
      role="button"
      tabindex="0"
      onclick={() => (showCustomScopeModal = false)}
      onkeydown={(e) => e.key === "Escape" && (showCustomScopeModal = false)}
    >
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="modal glass-strong card" onclick={(e) => e.stopPropagation()}>
        <div class="row-between" style="margin-bottom:1.25rem">
          <h2 class="section-title">New Custom Depressurizer Scope</h2>
          <button class="btn btn-ghost" onclick={() => (showCustomScopeModal = false)}>✕</button>
        </div>

        <div class="modal-body">
          <div class="field">
            <span>Scope Name</span>
            <input bind:value={newScopeName} placeholder="e.g. High-Playtime Favorites, VR Roguelikes…" />
          </div>

          <div class="row" style="gap:0.75rem">
            <div class="field" style="flex:1">
              <span>Field to Match</span>
              <select bind:value={newScopeField}>
                <option value="genre">Genre</option>
                <option value="tag">Community Tag</option>
                <option value="flag">Feature Flag (Co-op, VR)</option>
                <option value="developer">Developer</option>
                <option value="publisher">Publisher</option>
                <option value="platform">Platform (GOG, Steam…)</option>
                <option value="playtime">Playtime (Hours)</option>
                <option value="name">Title Name</option>
              </select>
            </div>

            <div class="field" style="flex:1">
              <span>Condition</span>
              <select bind:value={newScopeOperator}>
                <option value="contains">Contains</option>
                <option value="equals">Equals Exact</option>
                <option value="not_contains">Does Not Contain</option>
                <option value="less_than">Less Than (&lt;)</option>
                <option value="greater_than">Greater Than (&gt;)</option>
              </select>
            </div>
          </div>

          <div class="field">
            <span>Match Value</span>
            <input bind:value={newScopeValue} placeholder="e.g. Roguelike, Bethesda, 10…" />
          </div>

          <div class="field">
            <span>Target Steam Collection Name</span>
            <input bind:value={newScopeTarget} placeholder="e.g. Roguelikes, Bethesda RPGs…" />
          </div>
        </div>

        <div class="row-between modal-footer">
          <button class="btn btn-ghost" onclick={() => (showCustomScopeModal = false)}>Cancel</button>
          <button
            class="btn btn-primary"
            onclick={addCustomScope}
            disabled={!newScopeName.trim() || !newScopeTarget.trim()}
          >
            Add Scope
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
