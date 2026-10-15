<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error, settings, customNames } from "$lib/stores";
  import type { ArtworkSlotId, GameArtwork, SgdbAsset } from "$lib/types";
  import "$styles/artwork.scss";

  type SgdbKind = "grids" | "grids_portrait" | "grids_landscape" | "heroes" | "logos" | "icons";
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
  let webSearchAvailable = false;
  let fileSlot: ArtworkSlotId | null = null;
  let filePath = "";

  let hasKey = false;
$: hasKey = Boolean($settings?.steamgriddbApiKey);

  let shown: GameArtwork[] = [];
  let customSearchTerm = "";
  let dimensionChoice: ArtworkSlotId = "gridPortrait";
  let safeSearchEnabled = true;
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
    if (slot === "gridLandscape") return "grids_landscape";
    if (slot === "hero") return "heroes";
    if (slot === "logo") return "logos";
    if (slot === "icon") return "icons";
    return "grids_portrait";
  }

  function recommendedDims(slot: ArtworkSlotId): string {
    switch (slot) {
      case "gridPortrait":
        return "600×900 (2:3)";
      case "gridLandscape":
        return "920×430 (~2.14:1)";
      case "hero":
        return "1920×620 (~3.1:1)";
      case "logo":
        return "Transparent PNG";
      case "icon":
        return "256×256 (1:1)";
      default:
        return "";
    }
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
  
  // Fetch fallback artwork from alternative metadata providers
  async function fetchAlternativeAssets(kind: SgdbKind): Promise<SgdbAsset[]> {
    const assets: SgdbAsset[] = [];
    // 1. Steam Store metadata
    try {
      const metas = await api.fetchSteamStoreMetadata([selected!.appId]);
      const meta = metas[0];
      if (meta) {
        let url: string | undefined;
        switch (kind) {
          case "heroes":
            url = meta.heroImage ?? undefined;
            break;
          case "logos":
            url = meta.logoImage ?? undefined;
            break;
          case "grids":
          case "grids_portrait":
          case "grids_landscape":
            url = meta.headerImage ?? undefined;
            break;
          default:
            url = undefined;
        }
        if (url) {
          assets.push({ id: Date.now(), url, thumb: null, width: null, height: null, style: null, mime: null });
        }
      }
    } catch {}
    // 2. IGDB cover image fallback
    
    if (assets.length === 0 && $settings?.igdbClientId && $settings?.igdbClientSecret) {
      try {
        const games = await api.igdbSearch(selected!.name);
        if (games?.length) {
          const cover = games[0].cover?.url;
          if (cover) {
            const resolved = cover.replace("t_thumb", "t_original");
            assets.push({ id: Date.now() + 1, url: resolved, thumb: null, width: null, height: null, style: null, mime: null });
          }
        }
      } catch {}
    }


    // Additional providers (PCGamingWiki, Wikipedia, VNDB) can be added here.
    return assets;
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
      if (assets.length === 0) {
        // Attempt to fetch from alternative metadata providers.
        const fallback = await fetchAlternativeAssets(kindFor(slot));
        assets = fallback;
      }
      webSearchAvailable = assets.length === 0;
    } catch (e) {
      error.set(errorMessage(e));
      webSearchAvailable = false;
    } finally {
      assetsLoading = false;
    }
  }

  function toggleFile(slot: ArtworkSlotId) {
    fileSlot = fileSlot === slot ? null : slot;
    filePath = "";
  }

  // Expected dimensions for each artwork slot (width x height). Returns null for slots that accept any size.
  function getExpectedDimensions(slot: ArtworkSlotId): { width: number; height: number } | null {
    switch (slot) {
      case "gridPortrait":
        return { width: 600, height: 900 };
      case "gridLandscape":
        return { width: 920, height: 430 };
      case "hero":
        return { width: 1920, height: 620 };
      case "logo":
        return null; // any size PNG accepted
      case "icon":
        return null; // any size accepted (will never be rejected)
      default:
        return null;
    }
  }

  // Validate image dimensions (except slots that allow any size). Returns true if dimensions are within tolerance.
  async function validateImage(url: string, slot: ArtworkSlotId): Promise<boolean> {
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        const expected = getExpectedDimensions(slot);
        if (!expected) {
          resolve(true);
          return;
        }
        const tolerance = 20; // pixels tolerance
        const ok =
          Math.abs(img.width - expected.width) <= tolerance &&
          Math.abs(img.height - expected.height) <= tolerance;
        resolve(ok);
      };
      img.onerror = () => resolve(false);
      img.src = url;
    });
  }

  async function applyUrl(slot: ArtworkSlotId, url: string) {
    if (!selected) return;
    const trimmed = url.trim();
    if (!trimmed) {
      notice = "Error applying artwork: URL is empty";
      return;
    }
    // Strip query string to get a clean image URL
    const cleanUrl = trimmed.split("?")[0];

    // HEAD request to ensure URL is reachable and is an image
    try {
      const resp = await fetch(cleanUrl, { method: "HEAD" });
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const ct = resp.headers.get("content-type") ?? "";
      if (!ct.startsWith("image/")) throw new Error(`Content‑type ${ct} not an image`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      notice = `Error applying artwork: URL validation failed – ${msg}`;
      return;
    }

    // Dimension check (skip for icon slot)
    if (slot !== "icon") {
      const dimsOk = await validateImage(cleanUrl, slot);
      if (!dimsOk) {
        notice = `Error applying artwork: Image dimensions do not match required size for ${slot}.`;
        return;
      }
    }

    const appId = selected.appId;
    busy = true;
    busyLabel = "Applying artwork…";
    notice = null;
    try {
      const outcome = await api.applyArtworkFromUrl(appId, slot, cleanUrl);
      notice = `${labelOf(slot)} applied (${formatBytes(outcome.bytes)})${outcome.backup ? " · previous image backed up" : ""}.`;
      await refresh(appId);
    } catch (e) {
      const msg = errorMessage(e);
      notice = `Error applying artwork: ${msg}`;
      error.set(null);
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
      const msg = errorMessage(e);
      notice = `Error applying artwork: ${msg}`;
      error.set(null);
    } finally {
      busy = false;
    }
  }

  // UI helper to generate a web image search URL
  function webSearchUrl(): string {
    if (!selected) return "";
    const query = encodeURIComponent(`${selected.name} game artwork`);
    return `https://www.google.com/search?tbm=isch&q=${query}`;
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

  async function autoFetchArt() {
    busy = true;
    busyLabel = "Auto-fetching artwork for shortcuts…";
    notice = null;
    try {
      const count = await api.autoFetchArtworkForGames();
      notice = `Auto-fetched and applied ${count} artwork assets across library shortcuts.`;
      await refresh();
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>{$t("artwork.title")}</h1>
    <p class="muted">
      {$t("artwork.subtitle")}
    </p>
  </div>
  <div class="row" style="gap:0.5rem">
    <button class="btn" onclick={autoFetchArt} disabled={busy}>{$t("artwork.autoFetchAll")}</button>
    <button class="btn btn-ghost" onclick={() => refresh()} disabled={busy}>{$t("common.refresh")}</button>
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
            <div class="thumb thumb-{slot.slot}">
              {#if previews[previewKey(game.appId, slot.slot)]}
                <img src={previews[previewKey(game.appId, slot.slot)]} alt={slot.label} />
              {:else}
                <span class="thumb-empty">no art</span>
              {/if}
            </div>
            <div class="slot-body">
              <div class="row-between">
                <div class="slot-header-info">
                  <span class="slot-name">{slot.label}</span>
                  <span class="slot-spec">{recommendedDims(slot.slot)}</span>
                </div>
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
            <div class="browser-header-title">
              <div class="section-title" style="margin:0">SteamGridDB · {labelOf(current.slot)}</div>
              <span class="browser-spec">{recommendedDims(current.slot)}</span>
            </div>
            <button class="btn btn-ghost" onclick={() => (browser = null)}>Close</button>
<button class="btn btn-ghost" title="Search the web" onclick={() => window.open(webSearchUrl(), "_blank")} aria-label="Search the web">🌐</button>
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
              {#if webSearchAvailable}
                <button class="btn btn-ghost" onclick={() => window.open(webSearchUrl(), "_blank")}>Search the web</button>
              {/if}
          {:else}
            <div class="assets slot-{current.slot}">
              {#each assets as asset (asset.id)}
                <button
                  type="button"
                  class="asset"
                  onclick={() => applyUrl(current.slot, asset.url)}
                  disabled={busy}
                  title="Apply this artwork"
                >
                  <div class="asset-thumb-wrap aspect-{current.slot}">
                    <img src={asset.thumb || asset.url} alt="" loading="lazy" />
                    {#if asset.width && asset.height}
                      <span class="asset-size">{asset.width}×{asset.height}</span>
                    {/if}
                  </div>
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
