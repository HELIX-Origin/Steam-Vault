<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { error, settings } from "$lib/stores";
  import type { ArtworkSlotId, GameArtwork, SgdbAsset } from "$lib/types";

  type SgdbKind = "grids" | "heroes" | "logos" | "icons";
  type Filter = "all" | "missing" | "complete";

  let report: GameArtwork[] = [];
  let selected: GameArtwork | null = null;
  let gameSearch = "";
  let filter: Filter = "all";
  let busy = false;
  let busyLabel = "";
  let notice: string | null = null;
  let previews: Record<string, string | null> = {};
  let browser: { slot: ArtworkSlotId; kind: SgdbKind } | null = null;
  let assets: SgdbAsset[] = [];
  let assetsLoading = false;
  let fileSlot: ArtworkSlotId | null = null;
  let filePath = "";

  onMount(() => refresh());

  $: hasKey = Boolean($settings?.steamgriddbApiKey);
  $: shown = report.filter((game) => {
    if (filter === "missing" && game.applied === 5) return false;
    if (filter === "complete" && game.applied < 5) return false;
    const search = gameSearch.trim().toLowerCase();
    return !search || game.name.toLowerCase().includes(search);
  });
  $: withArt = report.filter((game) => game.applied > 0).length;

  function previewKey(appId: number, slot: ArtworkSlotId): string {
    return `${appId}:${slot}`;
  }

  function formatBytes(bytes: number): string {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB"];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
  }

  /** The SteamGridDB asset kind that fills a given Steam slot. */
  function kindFor(slot: ArtworkSlotId): SgdbKind {
    if (slot === "hero") return "heroes";
    if (slot === "logo") return "logos";
    if (slot === "icon") return "icons";
    return "grids";
  }

  function labelOf(slot: ArtworkSlotId): string {
    return selected?.slots.find((state) => state.slot === slot)?.label ?? slot;
  }

  async function refresh(keepAppId?: number) {
    busy = true;
    busyLabel = "Loading artwork…";
    try {
      report = await api.getArtwork();
      error.set(null);
      const target = keepAppId ?? selected?.appId;
      selected = target != null ? report.find((game) => game.appId === target) ?? null : null;
      if (selected) await loadPreviews(selected);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function loadPreviews(game: GameArtwork) {
    const next: Record<string, string | null> = {};
    await Promise.all(
      game.slots
        .filter((slot) => slot.present)
        .map(async (slot) => {
          const key = previewKey(game.appId, slot.slot);
          try {
            next[key] = await api.readArtwork(game.appId, slot.slot);
          } catch {
            next[key] = null;
          }
        })
    );
    previews = next;
  }

  async function selectGame(game: GameArtwork) {
    selected = game;
    browser = null;
    assets = [];
    fileSlot = null;
    filePath = "";
    await loadPreviews(game);
  }

  async function openBrowser(slot: ArtworkSlotId) {
    if (!selected) return;
    browser = { slot, kind: kindFor(slot) };
    assets = [];
    if (!hasKey) return;
    assetsLoading = true;
    try {
      assets = await api.steamGridDbForGame(selected.appId, selected.name, kindFor(slot));
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      assetsLoading = false;
    }
  }

  function toggleFile(slot: ArtworkSlotId) {
    fileSlot = fileSlot === slot ? null : slot;
    filePath = "";
  }

  async function applyUrl(slot: ArtworkSlotId, url: string) {
    if (!selected) return;
    const appId = selected.appId;
    busy = true;
    busyLabel = "Applying artwork…";
    notice = null;
    try {
      const outcome = await api.applyArtworkFromUrl(appId, slot, url);
      notice = `${labelOf(slot)} applied (${formatBytes(outcome.bytes)})${
        outcome.backup ? " · previous image backed up" : ""
      }.`;
      await refresh(appId);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function applyFile(slot: ArtworkSlotId) {
    if (!selected || !filePath.trim()) return;
    const appId = selected.appId;
    busy = true;
    busyLabel = "Applying artwork…";
    notice = null;
    try {
      const outcome = await api.applyArtworkFromFile(appId, slot, filePath.trim());
      notice = `${labelOf(slot)} applied from file (${formatBytes(outcome.bytes)}).`;
      fileSlot = null;
      filePath = "";
      await refresh(appId);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function removeSlot(slot: ArtworkSlotId) {
    if (!selected) return;
    const appId = selected.appId;
    busy = true;
    busyLabel = "Removing artwork…";
    notice = null;
    try {
      const outcome = await api.removeArtwork(appId, slot);
      notice = `${labelOf(slot)} removed${outcome.backup ? " (previous image backed up)" : ""}.`;
      await refresh(appId);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>Artwork</h1>
    <p class="muted">
      Apply grid, hero, logo, and icon art to any game — including non-Steam shortcuts — from
      SteamGridDB or a local file.
    </p>
  </div>
  <div class="row">
    <button class="btn btn-ghost" onclick={() => refresh()} disabled={busy}>Refresh</button>
  </div>
</div>

{#if !hasKey}
  <div class="glass-strong key-warn">
    <strong>No SteamGridDB API key.</strong>
    <span class="muted-sm">
      Add one in <a href="/settings">Settings</a> to browse artwork; you can still apply art from a
      local file.
    </span>
  </div>
{/if}

{#if notice}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <span class="muted-sm">{notice}</span>
  </div>
{/if}

{#if busy}
  <div class="glass-strong" style="padding:0.9rem 1rem;margin-bottom:1rem">
    <span class="muted-sm">{busyLabel || "Working…"}</span>
  </div>
{/if}

<div class="artwork-layout">
  <aside class="glass picker">
    <div class="picker-tools">
      <input bind:value={gameSearch} placeholder="Search games…" />
      <select bind:value={filter}>
        <option value="all">All games</option>
        <option value="missing">Missing art</option>
        <option value="complete">Complete</option>
      </select>
    </div>
    <span class="muted-sm">{withArt} of {report.length} games have custom art</span>
    <div class="pick-list">
      {#each shown as game (game.appId)}
        <button
          class="pick"
          class:active={selected?.appId === game.appId}
          onclick={() => selectGame(game)}
        >
          <span class="pick-name">{game.name}</span>
          <span class="pill" class:pill-dim={game.applied === 0}>{game.applied}/5</span>
        </button>
      {/each}
      {#if shown.length === 0}
        <p class="muted" style="padding:0.8rem">No games match.</p>
      {/if}
    </div>
  </aside>

  <section class="glass-strong editor">
    {#if selected}
      {@const game = selected}
      <div class="row-between">
        <div>
          <h2 class="section-title" style="margin:0">{game.name}</h2>
          <span class="muted-sm">App id {game.appId} · {game.applied} of 5 slots set</span>
        </div>
      </div>

      <div class="slots">
        {#each game.slots as slot (slot.slot)}
          <div class="slot">
            <div class="thumb">
              {#if previews[previewKey(game.appId, slot.slot)]}
                <img src={previews[previewKey(game.appId, slot.slot)]} alt={slot.label} />
              {:else}
                <span class="thumb-empty">no art</span>
              {/if}
            </div>
            <div class="slot-body">
              <div class="row-between">
                <span class="slot-name">{slot.label}</span>
                {#if slot.present}<span class="pill">{formatBytes(slot.bytes)}</span>{/if}
              </div>
              <div class="row slot-actions">
                <button class="btn btn-ghost" onclick={() => openBrowser(slot.slot)} disabled={busy}>
                  Browse…
                </button>
                <button class="btn btn-ghost" onclick={() => toggleFile(slot.slot)} disabled={busy}>
                  From file…
                </button>
                {#if slot.present}
                  <button class="btn btn-ghost btn-danger" onclick={() => removeSlot(slot.slot)} disabled={busy}>
                    Remove
                  </button>
                {/if}
              </div>
              {#if fileSlot === slot.slot}
                <div class="row slot-actions" style="margin-top:0.5rem">
                  <input bind:value={filePath} placeholder="C:\art\game.png" />
                  <button
                    class="btn btn-primary"
                    onclick={() => applyFile(slot.slot)}
                    disabled={busy || !filePath.trim()}
                  >
                    Apply
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      {#if browser}
        {@const current = browser}
        <div class="browser">
          <div class="row-between">
            <div class="section-title" style="margin:0">SteamGridDB · {labelOf(current.slot)}</div>
            <button class="btn btn-ghost" onclick={() => (browser = null)}>Close</button>
          </div>
          {#if !hasKey}
            <p class="muted-sm" style="margin:0.6rem 0 0">
              Add a SteamGridDB API key in <a href="/settings">Settings</a> to browse artwork.
            </p>
          {:else if assetsLoading}
            <p class="muted-sm" style="margin:0.6rem 0 0">Searching SteamGridDB…</p>
          {:else if assets.length === 0}
            <p class="muted-sm" style="margin:0.6rem 0 0">
              No matching artwork found. Try a local file instead.
            </p>
          {:else}
            <div class="assets">
              {#each assets as asset (asset.id)}
                <button
                  class="asset"
                  onclick={() => applyUrl(current.slot, asset.url)}
                  disabled={busy}
                  title="Apply this artwork"
                >
                  <img src={asset.thumb || asset.url} alt="" loading="lazy" />
                  {#if asset.width && asset.height}
                    <span class="asset-size">{asset.width}×{asset.height}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {:else}
      <p class="muted" style="padding:1.4rem">Select a game to manage its artwork.</p>
    {/if}
  </section>
</div>

<style>
  .artwork-layout {
    display: grid;
    grid-template-columns: minmax(260px, 320px) 1fr;
    gap: 14px;
    align-items: start;
  }

  @media (max-width: 900px) {
    .artwork-layout {
      grid-template-columns: 1fr;
    }
  }

  .key-warn {
    padding: 0.9rem 1rem;
    margin-bottom: 1rem;
    border-color: rgba(255, 196, 0, 0.35);
  }

  .picker {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding: 0.8rem;
    position: sticky;
    top: 0;
  }

  .picker-tools {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .pick-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 62vh;
    overflow-y: auto;
  }

  .pick {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5em 0.6em;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .pick:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .pick.active {
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.16), rgba(176, 108, 255, 0.14));
    border-color: var(--border-strong);
  }

  .pick-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .editor {
    padding: 1rem 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-height: 200px;
  }

  .slots {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .slot {
    display: grid;
    grid-template-columns: 132px 1fr;
    gap: 0.8rem;
    align-items: center;
    padding: 0.6rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .thumb {
    width: 132px;
    height: 74px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: rgba(255, 255, 255, 0.05);
    display: grid;
    place-items: center;
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .thumb-empty {
    font-size: 0.72rem;
    color: var(--text-mute);
  }

  .slot-body {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .slot-name {
    font-weight: 600;
  }

  .slot-actions {
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .browser {
    border-top: 1px solid var(--border);
    padding-top: 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .assets {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 0.5rem;
    max-height: 46vh;
    overflow-y: auto;
  }

  .asset {
    position: relative;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
    padding: 0;
    aspect-ratio: 3 / 2;
  }

  .asset:hover:not(:disabled) {
    border-color: var(--border-strong);
  }

  .asset img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .asset-size {
    position: absolute;
    right: 4px;
    bottom: 4px;
    font-size: 0.62rem;
    padding: 1px 5px;
    border-radius: 999px;
    background: rgba(2, 6, 12, 0.7);
    color: var(--text-dim);
  }
</style>
