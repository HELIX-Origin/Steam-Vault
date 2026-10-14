<script lang="ts">
  import { api, errorMessage } from "$lib/api";
  import { error, platformScan } from "$lib/stores";
  import type { Platform, PlatformGame } from "$lib/types";

  const labels: Record<Platform, string> = {
    steam: "Steam",
    gog: "GOG",
    epic: "Epic Games",
    itch: "itch.io",
    ubisoft: "Ubisoft Connect",
    ea: "EA",
    battlenet: "Battle.net",
    amazon: "Amazon Games",
    xbox: "Xbox",
    rockstar: "Rockstar",
    other: "Other"
  };

  let selected = new Set<string>();
  let filter: Platform | "all" = "all";
  let busy = false;
  let imported: { added: number; skipped: string[] } | null = null;

  function key(game: PlatformGame): string {
    return `${game.platform}|${game.installDir ?? game.exe ?? game.name}`.toLowerCase();
  }

  function toggle(game: PlatformGame) {
    const next = new Set(selected);
    const id = key(game);
    next.has(id) ? next.delete(id) : next.add(id);
    selected = next;
  }

  $: scan = $platformScan;
  $: available = scan?.games ?? [];
  $: shown = filter === "all" ? available : available.filter((game) => game.platform === filter);

  function selectVisible() {
    const next = new Set(selected);
    for (const game of shown) next.add(key(game));
    selected = next;
  }

  function clearSelection() {
    selected = new Set();
  }

  async function runScan() {
    busy = true;
    imported = null;
    try {
      platformScan.set(await api.detectPlatforms());
      selected = new Set();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function importSelected() {
    const chosen = available.filter((game) => selected.has(key(game)));
    if (chosen.length === 0) return;
    busy = true;
    try {
      const result = await api.importPlatformGames(chosen);
      imported = { added: result.added.length, skipped: result.skipped };
      selected = new Set();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>Platforms</h1>
    <p class="muted">
      Detect games installed by other launchers, then import them into Steam as non-Steam shortcuts.
    </p>
  </div>
  <div class="row">
    <button class="btn btn-primary" onclick={runScan} disabled={busy}>
      {busy ? "Scanning…" : "Scan for games"}
    </button>
  </div>
</div>

{#if imported}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <strong>{imported.added}</strong> game{imported.added === 1 ? "" : "s"} imported.
    {#if imported.skipped.length}
      <span class="muted-sm"> {imported.skipped.length} skipped (already present or no executable).</span>
    {/if}
  </div>
{/if}

{#if scan}
  <div class="grid cards">
    {#each scan.platforms as status (status.platform)}
      <button
        class="glass card"
        class:active={filter === status.platform}
        onclick={() => (filter = filter === status.platform ? "all" : status.platform)}
      >
        <div class="row-between">
          <span class="name">{status.label}</span>
          <span class="pill" class:pill-dim={status.count === 0}>{status.count}</span>
        </div>
        <span class="muted-sm">
          {#if status.available}
            {status.count} found
          {:else}
            no local data source
          {/if}
        </span>
        {#if status.notes}
          <span class="muted-sm note">{status.notes}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="glass-strong toolbar">
    <span class="muted-sm">
      {shown.length} game{shown.length === 1 ? "" : "s"} shown
      {#if filter !== "all"}· {labels[filter]}{/if}
    </span>
    <div class="spacer"></div>
    {#if filter !== "all"}
      <button class="btn btn-ghost" onclick={() => (filter = "all")}>Show all</button>
    {/if}
    <button class="btn" onclick={selectVisible} disabled={shown.length === 0}>Select shown</button>
    <button class="btn btn-ghost" onclick={clearSelection} disabled={selected.size === 0}>Clear</button>
    <button class="btn btn-primary" onclick={importSelected} disabled={busy || selected.size === 0}>
      Import {selected.size || ""} selected
    </button>
  </div>

  <div class="glass list">
    {#each shown as game (key(game))}
      <button class="row game" class:selected={selected.has(key(game))} onclick={() => toggle(game)}>
        <span class="check">{selected.has(key(game)) ? "✓" : ""}</span>
        <span class="name">
          {game.name}
          <span class="pill pill-accent2">{labels[game.platform]}</span>
        </span>
        <span class="muted-sm src">{game.source ?? ""}</span>
      </button>
    {/each}
    {#if shown.length === 0}
      <p class="muted" style="padding:1rem">
        Nothing to import here. Try another platform, or run a fresh scan.
      </p>
    {/if}
  </div>
{:else}
  <div class="glass" style="padding:2rem;text-align:center">
    <p class="muted">
      Run a scan to look for games installed by GOG, Epic Games, itch.io, Amazon Games, Ubisoft,
      EA, Battle.net, Rockstar, and more.
    </p>
    <p class="muted-sm">
      Steam Vault reads launcher manifests and the Windows Uninstall registry — it never scrapes
      storefronts or launches other launchers.
    </p>
  </div>
{/if}

<style>
  .cards {
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    margin-bottom: 1rem;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.9rem 1rem;
    text-align: left;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    border: 1px solid var(--border);
  }

  .card.active {
    border-color: var(--border-strong);
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.14), rgba(176, 108, 255, 0.12));
  }

  .card .name {
    font-weight: 600;
  }

  .note {
    opacity: 0.8;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.7rem 0.9rem;
    margin: 1rem 0;
    flex-wrap: wrap;
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

  .game {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.55em 0.7em;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    font: inherit;
    cursor: pointer;
    text-align: left;
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

  .src {
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
