<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error } from "$lib/stores";
  import type { LibraryEntry, StorageReport } from "$lib/types";

  let report: StorageReport | null = null;
  let busy = false;
  let newLibraryPath = "";
  let actionMessage: string | null = null;

  onMount(async () => {
    await refresh();
  });

  async function refresh() {
    busy = true;
    try {
      report = await api.getStorageReport();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function handleAddLibrary() {
    if (!newLibraryPath.trim()) return;
    busy = true;
    try {
      await api.addLibraryFolder(newLibraryPath.trim());
      newLibraryPath = "";
      actionMessage = "Library folder added successfully!";
      await refresh();
      setTimeout(() => (actionMessage = null), 4000);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function handleRemoveLibrary(path: string) {
    busy = true;
    try {
      await api.removeLibraryFolder(path);
      actionMessage = "Library folder removed!";
      await refresh();
      setTimeout(() => (actionMessage = null), 4000);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  $: libraries = report?.libraries ?? [];
  $: disks = report?.disks ?? [];
  $: totalLibraryBytes = libraries.reduce((sum, library) => sum + library.totalBytes, 0);
  $: totalDiskCapacity = disks.reduce((sum, d) => sum + d.total, 0);
  $: totalFreeDiskSpace = disks.reduce((sum, d) => sum + d.free, 0);
  $: totalGamesTracked = libraries.reduce((sum, lib) => sum + lib.gameCount, 0);

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
</script>

<div class="row-between" style="margin-bottom:1.25rem">
  <div>
    <h1>{$t("storage.title")}</h1>
    <p class="muted">{$t("storage.subtitle")}</p>
  </div>
  <div class="row" style="gap:0.5rem">
    {#if actionMessage}
      <span class="pill pill-glow">{actionMessage}</span>
    {/if}
    <button class="btn btn-ghost" onclick={refresh} disabled={busy}>{$t("common.refresh")}</button>
  </div>
</div>

{#if report}
  <!-- Storage KPI Overview Cards -->
  <div class="grid kpi-grid">
    <div class="glass card kpi-card">
      <span class="kpi-num">{formatBytes(totalLibraryBytes)}</span>
      <span class="kpi-label">Steam Games Footprint</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{formatBytes(totalFreeDiskSpace)}</span>
      <span class="kpi-label">Free Across {disks.length} Drives</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{libraries.length}</span>
      <span class="kpi-label">Registered Libraries</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{totalGamesTracked}</span>
      <span class="kpi-label">Games Installed</span>
    </div>
  </div>

  <!-- Connected Drives Section -->
  <div class="section-container">
    <div class="row-between section-head">
      <div>
        <h2 class="section-title">Connected Storage Drives</h2>
        <p class="muted-sm">Physical and logical storage partitions available on this system.</p>
      </div>
      <span class="muted-sm">{disks.length} drive{disks.length === 1 ? "" : "s"} detected</span>
    </div>
    <div class="grid drives">
      {#each disks as disk (disk.mount)}
        <div class="glass card drive-card">
          <div class="row-between">
            <span class="name">{disk.name?.trim() || disk.mount}</span>
            <span class="pill pill-accent">{disk.mount}</span>
          </div>
          <div class="bar">
            <span style="width:{usedFraction(disk.total, disk.free) * 100}%"></span>
          </div>
          <div class="row-between muted-sm">
            <span>{formatBytes(disk.total - disk.free)} used ({((disk.total - disk.free) / (disk.total || 1) * 100).toFixed(0)}%)</span>
            <span>{formatBytes(disk.free)} free of {formatBytes(disk.total)}</span>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Steam Library Folders Section -->
  <div class="section-container">
    <div class="row-between section-head">
      <div>
        <h2 class="section-title">Steam Library Folders</h2>
        <p class="muted-sm">
          Locations registered in Steam's <code>libraryfolders.vdf</code> where games are installed.
        </p>
      </div>
      <span class="muted-sm">{libraries.length} registered · {formatBytes(totalLibraryBytes)} total</span>
    </div>

    <!-- Add library folder toolbar -->
    <div class="glass-strong card add-library-bar">
      <span class="add-lib-label">Register Steam Library:</span>
      <input
        type="text"
        bind:value={newLibraryPath}
        placeholder="e.g. D:\SteamLibrary or /mnt/games/SteamLibrary"
        class="add-lib-input"
      />
      <button class="btn btn-primary" onclick={handleAddLibrary} disabled={busy || !newLibraryPath.trim()}>
        ＋ Add Library Folder
      </button>
    </div>

    <div class="grid cards">
      {#each libraries as entry (entry.path)}
        <div class="glass card lib-card">
          <div class="row-between">
            <span class="name">{entry.label?.trim() || `Library ${entry.index}`}</span>
            <div class="row" style="gap:0.35rem">
              {#if entry.isSteamRoot}<span class="pill pill-accent">Steam Root</span>{/if}
              {#if !entry.exists}<span class="pill pill-danger">Missing Folder</span>{/if}
              {#if !entry.isSteamRoot}
                <button
                  class="btn mini btn-ghost btn-danger"
                  onclick={() => handleRemoveLibrary(entry.path)}
                  title="Remove Library Registration"
                >
                  ✕
                </button>
              {/if}
            </div>
          </div>
          <span class="muted-sm path" title={entry.path}>{entry.path}</span>
          <div class="row-between muted-sm lib-stats">
            <span><strong>{entry.gameCount}</strong> games installed</span>
            <span><strong>{formatBytes(entry.totalBytes)}</strong></span>
          </div>
          {#if entry.diskFree != null}
            <div class="row-between muted-sm" style="font-size:0.75rem;padding-top:0.25rem;border-top:1px solid rgba(255,255,255,0.05)">
              <span>Drive Free: {formatBytes(entry.diskFree)}</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{:else}
  <div class="glass" style="padding:3rem;text-align:center;border-radius:var(--radius)">
    <p class="muted">Loading storage statistics and drive overview…</p>
  </div>
{/if}

<style>
  .kpi-grid {
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .kpi-card {
    display: flex;
    flex-direction: column;
    padding: 1.15rem;
    text-align: center;
    border-radius: var(--radius);
  }

  .kpi-num {
    font-size: 1.65rem;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: -0.02em;
  }

  .kpi-label {
    font-size: 0.76rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-top: 0.25rem;
  }

  .section-container {
    margin-bottom: 1.75rem;
  }

  .section-head {
    align-items: flex-end;
    margin-bottom: 0.85rem;
  }

  .drives {
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .cards {
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 1.15rem;
    border-radius: var(--radius);
    transition: transform 0.15s ease, border-color 0.16s ease;
  }

  .card:hover {
    transform: translateY(-2px);
    border-color: var(--border-strong);
  }

  .drive-card, .lib-card {
    gap: 0.6rem;
  }

  .card .name {
    font-weight: 650;
    font-size: 0.95rem;
  }

  .bar {
    height: 8px;
    border-radius: var(--radius-pill);
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
    margin: 0.2rem 0;
  }

  .bar span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    border-radius: var(--radius-pill);
    box-shadow: 0 0 10px rgba(56, 189, 248, 0.45);
  }

  .path {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.25);
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-family: monospace;
  }

  .lib-stats {
    padding-top: 0.25rem;
  }

  .add-library-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .add-lib-label {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--text-dim);
  }

  .add-lib-input {
    flex: 1;
    min-width: 240px;
  }

  .pill-danger {
    background: rgba(248, 113, 113, 0.16);
    border-color: rgba(248, 113, 113, 0.35);
    color: #fecaca;
  }

  .mini {
    padding: 0.15rem 0.45rem;
    font-size: 0.75rem;
  }
</style>
