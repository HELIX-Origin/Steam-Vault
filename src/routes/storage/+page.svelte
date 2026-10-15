<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { api, errorMessage } from "$lib/api";
  import { error, steamRunning } from "$lib/stores";
  import type {
    GamePlacement,
    LibraryEntry,
    MovePreview,
    StorageProgress,
    StorageReport
  } from "$lib/types";

  let report: StorageReport | null = null;
  let busy = false;
  let busyLabel = "";
  let newFolder = "";
  let gameSearch = "";
  let libraryFilter = "";
  let progress: StorageProgress | null = null;
  let notice: string | null = null;

  type MoveDialog =
    | { kind: "game"; appId: number; title: string; from: string; target: string; preview: MovePreview | null }
    | { kind: "library"; title: string; from: string; target: string; preview: MovePreview | null };

  let move: MoveDialog | null = null;

  let unlisten: (() => void) | null = null;

  onMount(async () => {
    unlisten = await listen<StorageProgress>("storage-progress", (event) => {
      progress = event.payload;
    });
    await refresh();
  });

  onDestroy(() => unlisten?.());

  async function refresh() {
    try {
      report = await api.getStorageReport();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  $: libraries = report?.libraries ?? [];
  $: disks = report?.disks ?? [];
  $: games = report?.games ?? [];
  $: shownGames = games.filter((game) => {
    if (libraryFilter && game.libraryPath !== libraryFilter) return false;
    const search = gameSearch.trim().toLowerCase();
    return !search || game.name.toLowerCase().includes(search);
  });
  $: totalLibraryBytes = libraries.reduce((sum, library) => sum + library.totalBytes, 0);
  $: moveTargets =
    move && move.kind === "game" ? libraries.filter((library) => library.path !== move?.from) : [];

  function formatBytes(bytes: number): string {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
  }

  function usedFraction(total: number, free: number): number {
    if (!total) return 0;
    return Math.min(1, Math.max(0, (total - free) / total));
  }

  function labelFor(entry: LibraryEntry): string {
    return entry.label?.trim() || entry.path;
  }

  async function addFolder() {
    const path = newFolder.trim();
    if (!path) return;
    busy = true;
    busyLabel = "Registering library folder…";
    notice = null;
    try {
      await api.addLibraryFolder(path);
      newFolder = "";
      notice = `Registered ${path}.`;
      await refresh();
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function removeFolder(entry: LibraryEntry) {
    busy = true;
    busyLabel = `Removing ${entry.path}…`;
    notice = null;
    try {
      await api.removeLibraryFolder(entry.path);
      notice = `Unregistered ${entry.path} (files left on disk).`;
      await refresh();
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  function openLibraryMove(entry: LibraryEntry) {
    move = { kind: "library", title: `Move library ${labelFor(entry)}`, from: entry.path, target: "", preview: null };
  }

  /** Clear any computed preview when the destination changes. */
  function clearPreview() {
    if (move) move.preview = null;
  }

  function openGameMove(game: GamePlacement) {
    const other = libraries.find((library) => library.path !== game.libraryPath);
    move = {
      kind: "game",
      appId: game.appId,
      title: `Move ${game.name}`,
      from: game.libraryPath,
      target: other?.path ?? "",
      preview: null
    };
  }

  async function runPreview() {
    if (!move || !move.target.trim()) return;
    busy = true;
    try {
      move.preview =
        move.kind === "game"
          ? await api.previewMoveGame(move.appId, move.target.trim())
          : await api.previewMoveLibrary(move.from, move.target.trim());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function confirmMove() {
    if (!move || !move.preview) return;
    busy = true;
    busyLabel = "Moving…";
    progress = null;
    notice = null;
    const dialog = move;
    try {
      const outcome =
        dialog.kind === "game"
          ? await api.moveGame(dialog.appId, dialog.target.trim())
          : await api.moveLibrary(dialog.from, dialog.target.trim());
      notice = `Moved ${formatBytes(outcome.bytes)} in ${outcome.files} file${outcome.files === 1 ? "" : "s"}${
        outcome.sameVolume ? " (same drive)" : ""
      }; updated ${outcome.vdfUpdated.length} library file(s).`;
      move = null;
      await refresh();
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
      progress = null;
    }
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>Storage</h1>
    <p class="muted">Inspect library folders and drives, register new ones, and move games between libraries.</p>
  </div>
  <div class="row">
    <button class="btn btn-ghost" onclick={refresh} disabled={busy}>Refresh</button>
  </div>
</div>

{#if $steamRunning}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem;border-color:rgba(255,107,129,0.4)">
    <strong>Steam is running.</strong>
    <span class="muted-sm"> Close Steam completely before adding, removing, or moving libraries.</span>
  </div>
{/if}

{#if notice}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <span class="muted-sm">{notice}</span>
  </div>
{/if}

{#if busy && progress}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <div class="row-between">
      <span>{progress.label}</span>
      <span class="muted-sm">
        {formatBytes(progress.done)} / {formatBytes(progress.total)}
      </span>
    </div>
    <div class="bar"><span style="width:{progress.total ? (progress.done / progress.total) * 100 : 0}%"></span></div>
  </div>
{:else if busy}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <span class="muted-sm">{busyLabel || "Working…"}</span>
  </div>
{/if}

{#if report}
  <div class="section-title">Drives</div>
  <div class="grid drives">
    {#each disks as disk (disk.mount)}
      <div class="glass card">
        <div class="row-between">
          <span class="name">{disk.name || disk.mount}</span>
          {#if disk.removable}<span class="pill pill-dim">removable</span>{/if}
        </div>
        <div class="bar"><span style="width:{usedFraction(disk.total, disk.free) * 100}%"></span></div>
        <span class="muted-sm">
          {formatBytes(disk.free)} free of {formatBytes(disk.total)} · {disk.mount}
        </span>
      </div>
    {/each}
  </div>

  <div class="row-between" style="margin:1.4rem 0 0.6rem">
    <div class="section-title" style="margin:0">Libraries</div>
    <span class="muted-sm">{libraries.length} registered · {formatBytes(totalLibraryBytes)} of games</span>
  </div>
  <div class="grid cards">
    {#each libraries as entry (entry.path)}
      <div class="glass card">
        <div class="row-between">
          <span class="name">{entry.label?.trim() || `Library ${entry.index}`}</span>
          <div class="row" style="gap:0.35rem">
            {#if entry.isSteamRoot}<span class="pill">Steam</span>{/if}
            {#if !entry.exists}<span class="pill pill-danger">missing</span>{/if}
          </div>
        </div>
        <span class="muted-sm path" title={entry.path}>{entry.path}</span>
        <span class="muted-sm">
          {entry.gameCount} game{entry.gameCount === 1 ? "" : "s"} · {formatBytes(entry.totalBytes)}
          {#if entry.diskFree != null} · {formatBytes(entry.diskFree)} free{/if}
        </span>
        <div class="row" style="gap:0.4rem;margin-top:0.2rem">
          <button class="btn btn-ghost" onclick={() => openLibraryMove(entry)} disabled={busy}>Move…</button>
          <button
            class="btn btn-ghost btn-danger"
            onclick={() => removeFolder(entry)}
            disabled={busy || entry.isSteamRoot || entry.gameCount > 0}
            title={entry.gameCount > 0 ? "Move its games out first" : ""}
          >
            Remove
          </button>
        </div>
      </div>
    {/each}
  </div>

  <div class="glass-strong" style="padding:0.9rem 1rem;margin:1rem 0 1.4rem">
    <div class="section-title" style="margin-bottom:0.5rem">Add a library folder</div>
    <div class="row">
      <input bind:value={newFolder} placeholder="e.g. D:\SteamLibrary" onkeydown={(e) => e.key === "Enter" && addFolder()} />
      <button class="btn btn-primary" onclick={addFolder} disabled={busy || !newFolder.trim()}>Add</button>
    </div>
    <p class="muted-sm" style="margin:0.6rem 0 0">
      The folder's <code>steamapps</code> subfolder is created if needed, then registered in Steam's
      <code>libraryfolders.vdf</code>.
    </p>
  </div>

  <div class="row-between" style="margin-bottom:0.6rem">
    <div class="section-title" style="margin:0">Games</div>
    <span class="muted-sm">{shownGames.length} of {games.length} shown</span>
  </div>
  <div class="glass-strong toolbar">
    <input bind:value={gameSearch} placeholder="Search games…" style="max-width:240px" />
    <select bind:value={libraryFilter} style="max-width:320px">
      <option value="">All libraries</option>
      {#each libraries as entry (entry.path)}
        <option value={entry.path}>{labelFor(entry)}</option>
      {/each}
    </select>
  </div>

  <div class="glass list">
    {#each shownGames as game (game.appId)}
      <div class="row game">
        <span class="name">
          {game.name}
          {#if !game.installed}<span class="pill pill-dim">not installed</span>{/if}
          {#if game.installed && !game.folderExists}<span class="pill pill-danger">folder missing</span>{/if}
        </span>
        <span class="muted-sm src" title={game.libraryPath}>
          {libraries.find((library) => library.path === game.libraryPath)?.label?.trim() || game.libraryPath}
        </span>
        <span class="muted-sm size">{formatBytes(game.sizeOnDisk)}</span>
        <button
          class="btn btn-ghost"
          onclick={() => openGameMove(game)}
          disabled={busy || !game.folderExists || libraries.length < 2}
        >
          Move…
        </button>
      </div>
    {/each}
    {#if shownGames.length === 0}
      <p class="muted" style="padding:1rem">No games match.</p>
    {/if}
  </div>
{:else}
  <div class="glass" style="padding:2rem;text-align:center">
    <p class="muted">Loading library storage…</p>
  </div>
{/if}

{#if move}
  <div class="overlay">
    <div class="glass-strong dialog">
      <h2 class="section-title">{move.title}</h2>
      <label class="field" style="display:block">
        <span class="muted-sm">{move.kind === "game" ? "Destination library" : "New location"}</span>
        {#if move.kind === "game"}
          <select bind:value={move.target} onchange={clearPreview}>
            {#each moveTargets as entry (entry.path)}
              <option value={entry.path}>{labelFor(entry)}</option>
            {/each}
          </select>
        {:else}
          <input bind:value={move.target} placeholder="e.g. E:\SteamLibrary" oninput={clearPreview} />
        {/if}
      </label>

      <div class="row" style="gap:0.5rem">
        <button class="btn" onclick={runPreview} disabled={busy || !move.target.trim()}>Preview</button>
        <button class="btn btn-primary" onclick={confirmMove} disabled={busy || !move.preview || !move.preview.fits}>
          Move
        </button>
        <div class="spacer"></div>
        <button class="btn btn-ghost" onclick={() => (move = null)} disabled={busy}>Cancel</button>
      </div>

      {#if move.preview}
        <div class="preview">
          <div class="row-between">
            <span class="muted-sm path" title={move.preview.from}>from {move.preview.from}</span>
          </div>
          <div class="row-between">
            <span class="muted-sm path" title={move.preview.to}>to {move.preview.to}</span>
          </div>
          <p class="muted-sm" style="margin:0.5rem 0 0">
            {formatBytes(move.preview.bytes)} to move ·
            {move.preview.sameVolume ? "same drive (instant)" : "across drives"}
            {#if move.preview.targetFree != null} · {formatBytes(move.preview.targetFree)} free at destination{/if}
          </p>
          {#if move.preview.warnings.length}
            {#each move.preview.warnings as warning}
              <p class="warn">{warning}</p>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .drives {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  }

  .cards {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.9rem 1rem;
  }

  .card .name {
    font-weight: 600;
  }

  .bar {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .bar span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
  }

  .path {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pill-danger {
    background: rgba(255, 107, 129, 0.14);
    border-color: rgba(255, 107, 129, 0.35);
    color: #ffd9df;
  }

  .toolbar {
    display: flex;
    gap: 0.6rem;
    padding: 0.7rem 0.9rem;
    margin-bottom: 0.8rem;
    flex-wrap: wrap;
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
    gap: 0.7rem;
    padding: 0.5em 0.7em;
    border-radius: var(--radius-sm);
  }

  .game:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .game .name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .game .src {
    max-width: 32%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game .size {
    width: 5.5rem;
    text-align: right;
    flex-shrink: 0;
  }

  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(2, 6, 12, 0.6);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    padding: 1.5rem;
    z-index: 20;
  }

  .dialog {
    width: min(560px, 100%);
    padding: 1.2rem 1.3rem;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .preview {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0.7rem 0.8rem;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .warn {
    color: var(--warn);
    font-size: 0.82rem;
    margin: 0.2rem 0 0;
  }
</style>
