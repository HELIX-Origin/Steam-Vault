<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, loading } from "$lib/stores";
  import type { AppMeta, Game, Platform } from "$lib/types";

  // Display & layout state
  let viewMode: "grid" | "list" = "grid";
  let sidebarOpen = true;

  // Filter & sort state
  let search = "";
  let sortBy: "name-asc" | "name-desc" | "playtime-desc" | "playtime-asc" | "recent-desc" | "recent-asc" | "size-desc" | "size-asc" = "name-asc";
  let installFilter: "all" | "installed" | "uninstalled" = "all";
  let platformFilter: Platform | "all" = "all";
  let collectionFilter: string | null = null;
  let onlyFavorites = false;
  let showHidden = false;

  // Multi-selection state
  let selected = new Set<number>();
  let targetCollection: string | null = null;
  let bulkPlatform = "";
  let busy = false;
  let syncing = false;

  // Per-game details / management sub-page state
  let selectedGame: Game | null = null;
  let gameMeta: AppMeta | null = null;
  let loadingMeta = false;
  let mediaImages: string[] = [];
  let selectedGalleryIdx = 0;
  let activeLightboxImg: string | null = null;
  let showManageModal = false;
  let actionFeedback: string | null = null;

  const platformLabels: Record<string, string> = {
    steam: "Steam",
    gog: "GOG",
    epic: "Epic Games",
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
    { id: "epic", label: "Epic Games" },
    { id: "itch", label: "itch.io" },
    { id: "ubisoft", label: "Ubisoft" },
    { id: "ea", label: "EA" },
    { id: "battlenet", label: "Battle.net" },
    { id: "amazon", label: "Amazon Games" },
    { id: "xbox", label: "Xbox" },
    { id: "rockstar", label: "Rockstar" },
    { id: "emulator", label: "Emulators" },
    { id: "other", label: "Other" }
  ];

  $: selectedShortcuts = $games.filter((g) => selected.has(g.appId) && g.kind === "shortcut");
  $: installedCount = $games.filter((g) => g.installed).length;
  $: uninstalledCount = $games.length - installedCount;
  $: favoritesCount = $games.filter((g) => g.favorite).length;

  function countForPlatform(p: Platform): number {
    return $games.filter((g) => g.platform === p).length;
  }

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

  // Filter and sort computation
  $: filtered = $games
    .filter((game) => {
      if (search && !game.name.toLowerCase().includes(search.toLowerCase())) return false;
      if (installFilter === "installed" && !game.installed) return false;
      if (installFilter === "uninstalled" && game.installed) return false;
      if (platformFilter !== "all" && game.platform !== platformFilter) return false;
      if (collectionFilter && !game.collections.includes(collectionFilter)) return false;
      if (onlyFavorites && !game.favorite) return false;
      if (!showHidden && game.hidden) return false;
      return true;
    })
    .sort((a, b) => {
      switch (sortBy) {
        case "name-asc":
          return a.name.localeCompare(b.name);
        case "name-desc":
          return b.name.localeCompare(a.name);
        case "playtime-desc":
          return (b.playtimeMinutes ?? 0) - (a.playtimeMinutes ?? 0);
        case "playtime-asc":
          return (a.playtimeMinutes ?? 0) - (b.playtimeMinutes ?? 0);
        case "recent-desc":
          return (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0);
        case "recent-asc":
          return (a.lastPlayed ?? 0) - (b.lastPlayed ?? 0);
        case "size-desc":
          return (b.sizeOnDisk ?? 0) - (a.sizeOnDisk ?? 0);
        case "size-asc":
          return (a.sizeOnDisk ?? 0) - (b.sizeOnDisk ?? 0);
        default:
          return 0;
      }
    });

  function toggleSelection(appId: number, e?: Event) {
    if (e) e.stopPropagation();
    const next = new Set(selected);
    next.has(appId) ? next.delete(appId) : next.add(appId);
    selected = next;
  }

  function selectAll() {
    selected = new Set(filtered.map((g) => g.appId));
  }

  function clearSelection() {
    selected = new Set();
  }

  async function assign(mode: "add" | "remove") {
    if (!targetCollection || selected.size === 0) return;
    busy = true;
    try {
      const col = $collections.find((c) => c.name === targetCollection);
      if (!col) throw new Error("Collection not found");
      await api.updateCollection({
        collectionId: col.id,
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

  async function setPlatform(game: Game, platform: string) {
    try {
      await api.setGamePlatform(game.appId, platform);
      game.platform = platform as Platform;
      games.set([...$games]);
      if (selectedGame && selectedGame.appId === game.appId) {
        selectedGame.platform = platform as Platform;
      }
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

  function formatSize(bytes: number | null | undefined): string {
    if (!bytes) return "—";
    const gb = bytes / 1024 ** 3;
    return gb >= 1 ? `${gb.toFixed(1)} GB` : `${(bytes / 1024 ** 2).toFixed(0)} MB`;
  }

  function formatPlaytime(minutes: number | null | undefined): string {
    if (!minutes) return "0 hours";
    const hours = (minutes / 60).toFixed(1);
    return `${hours} hrs`;
  }

  function formatLastPlayed(timestamp: number | null | undefined): string {
    if (!timestamp) return "Never";
    const d = new Date(timestamp * 1000);
    return d.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
  }

  function getCapsuleUrl(appId: number): string {
    return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_600x900.jpg`;
  }

  function getHeroUrl(appId: number): string {
    return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/library_hero.jpg`;
  }

  function getHeaderUrl(appId: number): string {
    return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`;
  }

  function getLogoUrl(appId: number): string {
    return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/logo.png`;
  }

  // Open extensive per-game details view
  async function openGameDetails(game: Game) {
    selectedGame = game;
    gameMeta = null;
    loadingMeta = true;
    selectedGalleryIdx = 0;
    actionFeedback = null;
    mediaImages = [];

    // Assemble up to 10 media gallery images for this game
    const defaultImages = [
      getHeroUrl(game.appId),
      getCapsuleUrl(game.appId),
      getHeaderUrl(game.appId),
      getLogoUrl(game.appId),
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_1.jpg`,
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_2.jpg`,
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_3.jpg`,
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_4.jpg`,
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_5.jpg`,
      `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/ss_6.jpg`
    ];
    mediaImages = defaultImages.slice(0, 10);

    try {
      if (game.kind === "steam" || game.appId < 0x80000000) {
        const metas = await api.fetchSteamStoreMetadata([game.appId]);
        if (metas.length > 0) {
          gameMeta = metas[0];
        }
      }
      if (!gameMeta && game.name) {
        const catalogMeta = await api.fetchOnlineGameEntry(game.name).catch(() => null);
        if (catalogMeta) {
          gameMeta = catalogMeta;
        }
      }
    } catch (e) {
      console.warn("Failed to fetch detailed metadata for game", e);
    } finally {
      loadingMeta = false;
    }
  }

  function closeGameDetails() {
    selectedGame = null;
    gameMeta = null;
    showManageModal = false;
    actionFeedback = null;
  }

  // Game action triggers
  async function handleLaunch(game: Game) {
    actionFeedback = "Launching game…";
    try {
      await api.launchGame(game.appId, game.launchUri, game.exe);
      actionFeedback = "Game launch signal sent!";
      setTimeout(() => (actionFeedback = null), 4000);
    } catch (e) {
      actionFeedback = `Launch error: ${errorMessage(e)}`;
    }
  }

  async function handleInstall(game: Game) {
    actionFeedback = "Opening game installer…";
    try {
      await api.installGame(game.appId, game.installUri);
      actionFeedback = "Install prompt opened!";
      setTimeout(() => (actionFeedback = null), 4000);
    } catch (e) {
      actionFeedback = `Install error: ${errorMessage(e)}`;
    }
  }

  async function handleUninstall(game: Game) {
    actionFeedback = "Opening uninstallation…";
    try {
      await api.uninstallGame(game.appId, game.uninstallUri);
      actionFeedback = "Uninstaller triggered!";
      setTimeout(() => (actionFeedback = null), 4000);
    } catch (e) {
      actionFeedback = `Uninstall error: ${errorMessage(e)}`;
    }
  }

  async function handleOpenFolder(game: Game) {
    const path = game.installDir || game.startDir;
    if (!path) {
      actionFeedback = "No install folder known for this game.";
      return;
    }
    actionFeedback = "Opening folder in File Explorer…";
    try {
      await api.openGameFolder(path);
      actionFeedback = "Folder opened!";
      setTimeout(() => (actionFeedback = null), 3000);
    } catch (e) {
      actionFeedback = `Could not open folder: ${errorMessage(e)}`;
    }
  }

  async function toggleFavorite(game: Game) {
    game.favorite = !game.favorite;
    games.set([...$games]);
  }
</script>

<!-- ========================================================================= -->
<!-- PER-GAME EXTENSIVE DETAILS / MANAGEMENT VIEW                               -->
<!-- ========================================================================= -->
{#if selectedGame}
  <div class="game-details-view">
    <!-- Sub-page Header bar -->
    <div class="details-topbar glass-strong">
      <div class="row" style="gap:0.75rem">
        <button class="btn btn-ghost" onclick={closeGameDetails} title="Back to Library">
          ← {$t("nav.library")}
        </button>
        <span class="muted">/</span>
        <h2 class="game-top-title">{selectedGame.name || `App ${selectedGame.appId}`}</h2>
        <span class="pill pill-accent2">{platformLabels[selectedGame.platform] || selectedGame.platform}</span>
        {#if selectedGame.installed}
          <span class="pill pill-ok">Installed</span>
        {:else}
          <span class="pill pill-dim">Not Installed</span>
        {/if}
      </div>

      <div class="row" style="gap:0.6rem">
        {#if actionFeedback}
          <span class="pill pill-glow feedback-pill">{actionFeedback}</span>
        {/if}

        {#if selectedGame.installed}
          <button class="btn btn-primary play-btn" onclick={() => handleLaunch(selectedGame!)}>
            ▶ Play
          </button>
        {:else}
          <button class="btn btn-primary install-btn" onclick={() => handleInstall(selectedGame!)}>
            ⬇ Install
          </button>
        {/if}

        <button class="btn btn-ghost" onclick={() => (showManageModal = true)} title="Manage Game Options">
          ⚙ Manage
        </button>

        <button
          class="btn btn-ghost fav-btn"
          class:is-fav={selectedGame.favorite}
          onclick={() => toggleFavorite(selectedGame!)}
          title="Toggle Favorite"
        >
          {selectedGame.favorite ? "★" : "☆"}
        </button>
      </div>
    </div>

    <!-- Hero banner -->
    <div class="details-hero glass">
      <div
        class="hero-backdrop"
        style="background-image: url('{getHeroUrl(selectedGame.appId)}'), url('{getHeaderUrl(selectedGame.appId)}');"
      ></div>
      <div class="hero-overlay"></div>
      <div class="hero-content">
        <img
          class="hero-capsule"
          src={getCapsuleUrl(selectedGame.appId)}
          alt={selectedGame.name}
          onerror={(e) => {
            (e.currentTarget as HTMLElement).style.display = 'none';
          }}
        />
        <div class="hero-meta">
          <h1 class="hero-title">{selectedGame.name || `App ${selectedGame.appId}`}</h1>
          {#if gameMeta?.shortDescription}
            <p class="hero-desc">{gameMeta.shortDescription}</p>
          {/if}
          <div class="row hero-tags">
            <span class="pill pill-accent">{platformLabels[selectedGame.platform]}</span>
            <span class="pill pill-dim">App ID: {selectedGame.appId}</span>
            {#if selectedGame.playtimeMinutes}
              <span class="pill pill-dim">Playtime: {formatPlaytime(selectedGame.playtimeMinutes)}</span>
            {/if}
            {#if selectedGame.lastPlayed}
              <span class="pill pill-dim">Last played: {formatLastPlayed(selectedGame.lastPlayed)}</span>
            {/if}
            {#if selectedGame.sizeOnDisk}
              <span class="pill pill-dim">{formatSize(selectedGame.sizeOnDisk)}</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content Layout: Gallery + Info Cards -->
    <div class="details-grid">
      <!-- 10-Image Media Gallery -->
      <section class="glass-strong card media-gallery-card">
        <div class="row-between" style="margin-bottom:0.75rem">
          <div>
            <h3 class="section-title">Media Gallery</h3>
            <p class="muted-sm">10-image screenshot & artwork showcase. Click any image to view full resolution.</p>
          </div>
          <span class="muted-sm">{selectedGalleryIdx + 1} / {mediaImages.length}</span>
        </div>

        <!-- Featured Media Display -->
        <div
          class="featured-media"
          role="button"
          tabindex="0"
          onclick={() => (activeLightboxImg = mediaImages[selectedGalleryIdx])}
          onkeydown={(e) => e.key === "Enter" && (activeLightboxImg = mediaImages[selectedGalleryIdx])}
        >
          <img
            src={mediaImages[selectedGalleryIdx]}
            alt="Game screenshot"
            onerror={(e) => {
              (e.currentTarget as HTMLImageElement).src = getHeaderUrl(selectedGame!.appId);
            }}
          />
          <div class="media-zoom-hint">🔍 Expand to Fullscreen</div>
        </div>

        <!-- 10 Thumbnail Strip -->
        <div class="gallery-thumbnails">
          {#each mediaImages as img, idx}
            <button
              class="thumb-btn"
              class:active={selectedGalleryIdx === idx}
              onclick={() => (selectedGalleryIdx = idx)}
            >
              <img
                src={img}
                alt="Thumbnail {idx + 1}"
                onerror={(e) => {
                  (e.currentTarget as HTMLImageElement).src = getHeaderUrl(selectedGame!.appId);
                }}
              />
            </button>
          {/each}
        </div>
      </section>

      <!-- Right column: Info Cards -->
      <div class="details-sidebar">
        <!-- Meta Links Card -->
        <div class="glass-strong card">
          <h3 class="section-title">Meta & Reference Links</h3>
          <div class="meta-links-list">
            {#if selectedGame.kind === "steam" || selectedGame.platform === "steam"}
              <a
                class="meta-link"
                href={`https://store.steampowered.com/app/${selectedGame.appId}`}
                target="_blank"
                rel="noreferrer"
              >
                <span class="meta-icon">🌐</span> Steam Store Page
              </a>
              <a
                class="meta-link"
                href={`https://steamdb.info/app/${selectedGame.appId}`}
                target="_blank"
                rel="noreferrer"
              >
                <span class="meta-icon">📊</span> SteamDB Overview
              </a>
              <a
                class="meta-link"
                href={`https://www.protondb.com/app/${selectedGame.appId}`}
                target="_blank"
                rel="noreferrer"
              >
                <span class="meta-icon">🐧</span> ProtonDB Compatibility
              </a>
            {/if}
            {#if selectedGame.platform === "gog" && selectedGame.platformId}
              <a
                class="meta-link"
                href={`https://www.gog.com/game/${selectedGame.platformId}`}
                target="_blank"
                rel="noreferrer"
              >
                <span class="meta-icon">🛍️</span> GOG Store Page
              </a>
            {/if}
            <a
              class="meta-link"
              href={`https://www.pcgamingwiki.com/wiki/Special:Search?search=${encodeURIComponent(selectedGame.name)}`}
              target="_blank"
              rel="noreferrer"
            >
              <span class="meta-icon">🔧</span> PCGamingWiki Fixes & Settings
            </a>
            <a
              class="meta-link"
              href={`https://en.wikipedia.org/wiki/Special:Search?search=${encodeURIComponent(selectedGame.name)}`}
              target="_blank"
              rel="noreferrer"
            >
              <span class="meta-icon">📖</span> Wikipedia Article
            </a>
            <a
              class="meta-link"
              href={`https://raw.githubusercontent.com/HELIX-Origin/Steam-Vault-Catalogue/main/games/${encodeURIComponent(selectedGame.name.toLowerCase().replace(/[^a-z0-9]+/g, "-"))}.json`}
              target="_blank"
              rel="noreferrer"
            >
              <span class="meta-icon">🗂️</span> Community Catalogue Record
            </a>
          </div>
        </div>

        <!-- Details Card -->
        <div class="glass-strong card">
          <h3 class="section-title">Game Information</h3>
          <div class="info-table">
            <div class="info-row">
              <span class="muted">Platform:</span>
              <span>{platformLabels[selectedGame.platform] || selectedGame.platform}</span>
            </div>
            {#if gameMeta?.developers && gameMeta.developers.length > 0}
              <div class="info-row">
                <span class="muted">Developers:</span>
                <span>{gameMeta.developers.join(", ")}</span>
              </div>
            {/if}
            {#if gameMeta?.publishers && gameMeta.publishers.length > 0}
              <div class="info-row">
                <span class="muted">Publishers:</span>
                <span>{gameMeta.publishers.join(", ")}</span>
              </div>
            {/if}
            {#if gameMeta?.releaseDate || gameMeta?.releaseYear}
              <div class="info-row">
                <span class="muted">Release Date:</span>
                <span>{gameMeta.releaseDate || gameMeta.releaseYear}</span>
              </div>
            {/if}
            {#if gameMeta?.userScore}
              <div class="info-row">
                <span class="muted">Metascore:</span>
                <span class="pill pill-accent">{gameMeta.userScore} / 100</span>
              </div>
            {/if}
            <div class="info-row">
              <span class="muted">Storage:</span>
              <span>{formatSize(selectedGame.sizeOnDisk)}</span>
            </div>
            {#if selectedGame.installDir}
              <div class="info-row full-width">
                <span class="muted">Install Folder:</span>
                <span class="code-val">{selectedGame.installDir}</span>
              </div>
            {/if}
            {#if selectedGame.exe}
              <div class="info-row full-width">
                <span class="muted">Executable:</span>
                <span class="code-val">{selectedGame.exe}</span>
              </div>
            {/if}
          </div>

          {#if gameMeta?.genres && gameMeta.genres.length > 0}
            <div style="margin-top:0.75rem">
              <span class="muted-sm" style="display:block;margin-bottom:0.35rem">Genres:</span>
              <div class="row" style="gap:0.35rem;flex-wrap:wrap">
                {#each gameMeta.genres as genre}
                  <span class="pill pill-dim">{genre}</span>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <!-- Playtime Tracking Card -->
        <div class="glass-strong card">
          <h3 class="section-title">Playtime Tracking</h3>
          <div class="playtime-stats">
            <div class="stat-box">
              <span class="stat-number">{formatPlaytime(selectedGame.playtimeMinutes)}</span>
              <span class="stat-label">Total Playtime</span>
            </div>
            <div class="stat-box">
              <span class="stat-number">{formatLastPlayed(selectedGame.lastPlayed)}</span>
              <span class="stat-label">Last Played</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Management Sub-Window Modal -->
  {#if showManageModal}
    <div
      class="modal-backdrop"
      role="button"
      tabindex="0"
      onclick={() => (showManageModal = false)}
      onkeydown={(e) => e.key === "Escape" && (showManageModal = false)}
    >
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="modal glass-strong card" onclick={(e) => e.stopPropagation()}>
        <div class="row-between modal-head">
          <h2 class="section-title">Manage: {selectedGame.name}</h2>
          <button class="btn btn-ghost" onclick={() => (showManageModal = false)}>✕</button>
        </div>

        <div class="modal-body">
          <!-- Folder Management -->
          <div class="manage-group">
            <span class="group-label">Local Files & Installation</span>
            <div class="row" style="gap:0.5rem">
              <button
                class="btn"
                onclick={() => handleOpenFolder(selectedGame!)}
                disabled={!selectedGame.installDir && !selectedGame.startDir}
              >
                📂 Open Game Folder
              </button>
              {#if selectedGame.installed}
                <button class="btn btn-danger" onclick={() => handleUninstall(selectedGame!)}>
                  🗑️ Uninstall Game
                </button>
              {:else}
                <button class="btn btn-primary" onclick={() => handleInstall(selectedGame!)}>
                  ⬇️ Install Game
                </button>
              {/if}
            </div>
          </div>

          <!-- Platform assignment -->
          <div class="manage-group">
            <span class="group-label">Storefront Platform Tag</span>
            <select
              class="platform-select-modal"
              value={selectedGame.platform}
              onchange={(e) => setPlatform(selectedGame!, e.currentTarget.value)}
            >
              {#each platformOptions as opt}
                <option value={opt.id}>{opt.label}</option>
              {/each}
            </select>
            <p class="muted-sm" style="margin-top:0.35rem">
              Assigns this game to a specific storefront catalogue for grouping and metadata.
            </p>
          </div>

          <!-- Collections Assignment -->
          <div class="manage-group">
            <span class="group-label">Category / Collection Assignment</span>
            <div class="row" style="gap:0.5rem">
              <select bind:value={targetCollection} style="flex:1">
                {#each $collections as col (col.id)}
                  <option value={col.name}>{col.name}</option>
                {/each}
              </select>
              <button
                class="btn btn-primary"
                onclick={() => {
                  selected = new Set([selectedGame!.appId]);
                  assign("add");
                }}
              >
                Add
              </button>
              <button
                class="btn btn-danger"
                onclick={() => {
                  selected = new Set([selectedGame!.appId]);
                  assign("remove");
                }}
              >
                Remove
              </button>
            </div>
          </div>

          <!-- Artwork management link -->
          <div class="manage-group">
            <span class="group-label">Artwork Customization</span>
            <div class="row" style="gap:0.5rem">
              <a class="btn" href="/artwork">
                🎨 Open in Artwork Manager
              </a>
            </div>
          </div>

          <!-- Launch specifics -->
          <div class="manage-group">
            <span class="group-label">Launch Protocol & Parameters</span>
            <div class="info-table">
              <div class="info-row full-width">
                <span class="muted">Launch URI:</span>
                <span class="code-val">{selectedGame.launchUri || `steam://run/${selectedGame.appId}`}</span>
              </div>
              {#if selectedGame.installUri}
                <div class="info-row full-width">
                  <span class="muted">Install URI:</span>
                  <span class="code-val">{selectedGame.installUri}</span>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <div class="modal-footer row-between">
          <span class="muted-sm">Changes take effect immediately.</span>
          <button class="btn btn-primary" onclick={() => (showManageModal = false)}>Done</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Fullscreen Lightbox Modal -->
  {#if activeLightboxImg}
    <div
      class="lightbox-overlay"
      role="button"
      tabindex="0"
      onclick={() => (activeLightboxImg = null)}
      onkeydown={(e) => e.key === "Escape" && (activeLightboxImg = null)}
    >
      <img src={activeLightboxImg} alt="Enlarged screenshot" class="lightbox-img" />
      <button class="lightbox-close" onclick={() => (activeLightboxImg = null)}>✕</button>
    </div>
  {/if}

<!-- ========================================================================= -->
<!-- MAIN REDESIGNED LIBRARY VIEW (GRID + RIGHT SIDEBAR)                       -->
<!-- ========================================================================= -->
{:else}
  <div class="library-shell">
    <div class="library-main">
      <!-- Top header -->
      <div class="row-between library-header">
        <div>
          <h1>{$t("nav.library")}</h1>
          <p class="muted">
            {$games.length} owned · {installedCount} installed · {filtered.length} shown · {selected.size} selected
          </p>
        </div>
        <div class="row" style="gap:0.5rem">
          <button class="btn" onclick={syncOwned} disabled={syncing}>
            {syncing ? $t("common.refreshing") : "Sync owned games"}
          </button>
          <button class="btn" onclick={selectAll}>{$t("common.all")}</button>
          <button class="btn btn-ghost" onclick={clearSelection} disabled={selected.size === 0}>
            {$t("common.clear")}
          </button>
          <button
            class="btn btn-ghost sidebar-toggle-btn"
            class:active={sidebarOpen}
            onclick={() => (sidebarOpen = !sidebarOpen)}
            title="Toggle Filter & Sort Sidebar"
          >
            ⚙ Filters
          </button>
        </div>
      </div>

      <!-- Bulk actions bar when items are selected -->
      {#if selected.size > 0}
        <div class="glass-strong bulk-bar">
          <span class="pill pill-accent">{selected.size} selected</span>
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
            {#each $collections as col (col.id)}
              <option value={col.name}>{col.name}</option>
            {/each}
          </select>
          <button class="btn btn-primary" disabled={busy} onclick={() => assign("add")}>{$t("common.add")}</button>
          <button class="btn btn-danger" disabled={busy} onclick={() => assign("remove")}>{$t("common.remove")}</button>
        </div>
      {/if}

      <!-- Main Game View -->
      {#if $loading}
        <p class="muted" style="padding:2rem">{$t("common.loading")}</p>
      {:else if viewMode === "grid"}
        <!-- GRID DISPLAY (Default & Primary Layout) -->
        <div class="games-grid">
          {#each filtered as game (game.kind + "-" + game.appId)}
            <div
              class="game-card glass"
              class:selected={selected.has(game.appId)}
              class:uninstalled={!game.installed}
              role="button"
              tabindex="0"
              onclick={() => openGameDetails(game)}
              onkeydown={(e) => e.key === "Enter" && openGameDetails(game)}
            >
              <!-- Card Cover Artwork -->
              <div class="card-cover-container">
                <img
                  class="card-cover"
                  src={getCapsuleUrl(game.appId)}
                  alt={game.name}
                  loading="lazy"
                  onerror={(e) => {
                    const target = e.currentTarget as HTMLImageElement;
                    target.style.display = 'none';
                    if (target.parentElement) {
                      target.parentElement.classList.add('has-fallback');
                    }
                  }}
                />
                <div class="card-cover-fallback">
                  <span class="fallback-icon">🎮</span>
                  <span class="fallback-title">{game.name || `App ${game.appId}`}</span>
                </div>

                <!-- Hover action bar -->
                <div class="card-hover-overlay">
                  {#if game.installed}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <button
                      class="card-play-btn"
                      title="Launch Game"
                      onclick={(e) => {
                        e.stopPropagation();
                        handleLaunch(game);
                      }}
                    >
                      ▶
                    </button>
                  {:else}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <button
                      class="card-play-btn install"
                      title="Install Game"
                      onclick={(e) => {
                        e.stopPropagation();
                        handleInstall(game);
                      }}
                    >
                      ⬇
                    </button>
                  {/if}
                </div>

                <!-- Select checkbox -->
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="card-check"
                  class:checked={selected.has(game.appId)}
                  onclick={(e) => toggleSelection(game.appId, e)}
                  title="Select for bulk actions"
                >
                  {selected.has(game.appId) ? "✓" : ""}
                </div>

                <!-- Top badges -->
                <div class="card-badges">
                  <span class="pill pill-platform">{platformLabels[game.platform] || game.platform}</span>
                  {#if !game.installed}
                    <span class="pill pill-uninstalled">Cloud</span>
                  {/if}
                </div>
              </div>

              <!-- Card Bottom Info -->
              <div class="card-info">
                <div class="card-title" title={game.name}>{game.name || `App ${game.appId}`}</div>
                <div class="card-meta">
                  <span class="muted-sm">{formatPlaytime(game.playtimeMinutes)}</span>
                  {#if game.favorite}<span class="card-fav">★</span>{/if}
                </div>
              </div>
            </div>
          {/each}
          {#if filtered.length === 0}
            <div class="empty-state glass">
              <p class="muted">No games match the current search or filters.</p>
            </div>
          {/if}
        </div>
      {:else}
        <!-- LIST DISPLAY (Alternate Layout) -->
        <div class="glass list-view">
          {#each filtered as game (game.kind + "-" + game.appId)}
            <div
              class="row game-list-row"
              class:selected={selected.has(game.appId)}
              role="button"
              tabindex="0"
              onclick={() => openGameDetails(game)}
              onkeydown={(e) => e.key === "Enter" && openGameDetails(game)}
            >
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <span
                class="check"
                onclick={(e) => toggleSelection(game.appId, e)}
              >
                {selected.has(game.appId) ? "✓" : ""}
              </span>

              <img
                src={getHeaderUrl(game.appId)}
                alt=""
                class="list-thumb"
                onerror={(e) => ((e.currentTarget as HTMLElement).style.display = 'none')}
              />

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
                {:else}
                  <span class="pill pill-dim">{platformLabels[game.platform]}</span>
                {/if}
                {#if !game.installed}<span class="pill pill-dim">not installed</span>{/if}
                {#if game.favorite}<span class="pill">★</span>{/if}
              </span>

              <span class="tags">
                {#each game.collections as col}
                  <span class="pill pill-dim">{col}</span>
                {/each}
              </span>

              <span class="muted-sm playtime-col">{formatPlaytime(game.playtimeMinutes)}</span>
              <span class="muted-sm size-col">{formatSize(game.sizeOnDisk)}</span>

              <!-- Action buttons -->
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div class="row" style="gap:0.35rem" onclick={(e) => e.stopPropagation()}>
                {#if game.installed}
                  <button class="btn mini btn-primary" onclick={() => handleLaunch(game)}>Play</button>
                {:else}
                  <button class="btn mini" onclick={() => handleInstall(game)}>Install</button>
                {/if}
              </div>
            </div>
          {/each}
          {#if filtered.length === 0}
            <p class="muted" style="padding:1.5rem">No games match the current filters.</p>
          {/if}
        </div>
      {/if}
    </div>

    <!-- ========================================================================= -->
    <!-- RIGHT SIDEBAR FOR SORTING & FILTERING                                      -->
    <!-- ========================================================================= -->
    {#if sidebarOpen}
      <aside class="library-sidebar glass-strong">
        <div class="sidebar-head row-between">
          <span class="sidebar-title">Filters & Options</span>
          <div class="view-switch-btns">
            <button
              class="view-btn"
              class:active={viewMode === "grid"}
              onclick={() => (viewMode = "grid")}
              title="Grid View"
            >
              ▦
            </button>
            <button
              class="view-btn"
              class:active={viewMode === "list"}
              onclick={() => (viewMode = "list")}
              title="List View"
            >
              ☰
            </button>
          </div>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Search Library</span>
          <div class="search-wrap">
            <input
              type="text"
              placeholder="Search by title…"
              bind:value={search}
              class="search-input"
            />
            {#if search}
              <button class="search-clear-btn" onclick={() => (search = "")}>✕</button>
            {/if}
          </div>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Sort Games By</span>
          <select bind:value={sortBy} class="sidebar-select">
            <option value="name-asc">Name (A → Z)</option>
            <option value="name-desc">Name (Z → A)</option>
            <option value="playtime-desc">Playtime (Most Played)</option>
            <option value="playtime-asc">Playtime (Least Played)</option>
            <option value="recent-desc">Last Played (Recent First)</option>
            <option value="recent-asc">Last Played (Oldest First)</option>
            <option value="size-desc">Size on Disk (Largest First)</option>
            <option value="size-asc">Size on Disk (Smallest First)</option>
          </select>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Installation Status</span>
          <div class="status-pill-group">
            <button
              class="status-pill"
              class:active={installFilter === "all"}
              onclick={() => (installFilter = "all")}
            >
              All ({$games.length})
            </button>
            <button
              class="status-pill"
              class:active={installFilter === "installed"}
              onclick={() => (installFilter = "installed")}
            >
              Installed ({installedCount})
            </button>
            <button
              class="status-pill"
              class:active={installFilter === "uninstalled"}
              onclick={() => (installFilter = "uninstalled")}
            >
              Cloud ({uninstalledCount})
            </button>
          </div>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Storefront Platform</span>
          <div class="platform-filter-list">
            <button
              class="platform-filter-item"
              class:active={platformFilter === "all"}
              onclick={() => (platformFilter = "all")}
            >
              <span>All Platforms</span>
              <span class="count-tag">{$games.length}</span>
            </button>
            {#each platformOptions as opt}
              {@const cnt = countForPlatform(opt.id)}
              {#if cnt > 0}
                <button
                  class="platform-filter-item"
                  class:active={platformFilter === opt.id}
                  onclick={() => (platformFilter = opt.id)}
                >
                  <span>{opt.label}</span>
                  <span class="count-tag">{cnt}</span>
                </button>
              {/if}
            {/each}
          </div>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Category / Collection</span>
          <select bind:value={collectionFilter} class="sidebar-select">
            <option value={null}>All Categories ({$collections.length})</option>
            {#each $collections as col (col.id)}
              <option value={col.name}>{col.name}</option>
            {/each}
          </select>
        </div>

        <div class="sidebar-section">
          <span class="sidebar-label">Quick Toggles</span>
          <div class="toggle-list">
            <label class="toggle-item">
              <input type="checkbox" bind:checked={onlyFavorites} />
              <span>★ Favorites only ({favoritesCount})</span>
            </label>
            <label class="toggle-item">
              <input type="checkbox" bind:checked={showHidden} />
              <span>👁 Show hidden games</span>
            </label>
          </div>
        </div>

        <div class="sidebar-footer">
          <span class="muted-sm">Showing {filtered.length} of {$games.length} games</span>
        </div>
      </aside>
    {/if}
  </div>
{/if}

<style>
  /* Base Layout */
  .library-shell {
    display: flex;
    gap: 1rem;
    height: 100%;
    align-items: flex-start;
  }

  .library-main {
    flex: 1;
    min-width: 0;
  }

  .library-header {
    margin-bottom: 1rem;
  }

  .bulk-bar {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.65rem 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
    border-radius: var(--radius-sm);
  }

  .spacer {
    flex: 1;
  }

  /* Right Sidebar */
  .library-sidebar {
    width: 280px;
    flex-shrink: 0;
    padding: 1rem;
    border-radius: var(--radius);
    display: flex;
    flex-direction: column;
    gap: 1.15rem;
    position: sticky;
    top: 0;
    max-height: calc(100vh - 80px);
    overflow-y: auto;
  }

  .sidebar-head {
    align-items: center;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .sidebar-title {
    font-weight: 650;
    font-size: 0.95rem;
    letter-spacing: -0.01em;
  }

  .view-switch-btns {
    display: flex;
    gap: 2px;
    background: rgba(0, 0, 0, 0.3);
    padding: 2px;
    border-radius: var(--radius-sm);
  }

  .view-btn {
    border: none;
    background: transparent;
    color: var(--text-dim);
    padding: 0.2rem 0.5rem;
    cursor: pointer;
    border-radius: 4px;
    font-size: 0.85rem;
    transition: all 0.15s ease;
  }

  .view-btn.active,
  .view-btn:hover {
    background: var(--panel-hover);
    color: var(--text);
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .sidebar-label {
    font-size: 0.76rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-dim);
  }

  .search-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 100%;
    padding-right: 1.8rem;
  }

  .search-clear-btn {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: var(--text-mute);
    cursor: pointer;
    font-size: 0.75rem;
  }

  .sidebar-select {
    width: 100%;
  }

  .status-pill-group {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
  }

  .status-pill {
    border: 1px solid var(--border-subtle);
    background: rgba(0, 0, 0, 0.2);
    color: var(--text-dim);
    font-size: 0.72rem;
    padding: 0.35rem 0.2rem;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: center;
    transition: all 0.15s ease;
  }

  .status-pill.active,
  .status-pill:hover {
    border-color: var(--accent);
    background: rgba(56, 189, 248, 0.15);
    color: var(--text);
  }

  .platform-filter-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 220px;
    overflow-y: auto;
    padding-right: 2px;
  }

  .platform-filter-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-dim);
    padding: 0.35rem 0.6rem;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s ease;
  }

  .platform-filter-item:hover {
    background: var(--panel-hover);
    color: var(--text);
  }

  .platform-filter-item.active {
    background: rgba(56, 189, 248, 0.16);
    border-color: var(--border-glow);
    color: var(--text);
    font-weight: 600;
  }

  .count-tag {
    font-size: 0.7rem;
    background: rgba(255, 255, 255, 0.08);
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-pill);
    color: var(--text-mute);
  }

  .toggle-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .toggle-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.82rem;
    cursor: pointer;
    color: var(--text-dim);
  }

  .toggle-item input {
    width: auto;
    cursor: pointer;
  }

  .sidebar-footer {
    padding-top: 0.5rem;
    border-top: 1px solid var(--border-subtle);
    text-align: center;
  }

  /* ========================================================================= */
  /* GRID VIEW STYLES                                                          */
  /* ========================================================================= */
  .games-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 1.15rem;
    padding-bottom: 2rem;
  }

  .game-card {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius);
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.18s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.18s ease, border-color 0.18s ease;
    border: 1px solid var(--border-subtle);
    background: rgba(14, 25, 44, 0.45);
    position: relative;
    user-select: none;
  }

  .game-card:hover {
    transform: translateY(-4px);
    border-color: var(--border-glow);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.5), 0 0 16px rgba(56, 189, 248, 0.2);
  }

  .game-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent), 0 12px 30px rgba(0, 0, 0, 0.5);
  }

  .game-card.uninstalled .card-cover {
    filter: saturate(0.65) brightness(0.85);
  }

  .card-cover-container {
    position: relative;
    width: 100%;
    aspect-ratio: 600 / 900;
    overflow: hidden;
    background: linear-gradient(145deg, #0f1c32, #08101e);
  }

  .card-cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 0.25s ease;
  }

  .game-card:hover .card-cover {
    transform: scale(1.04);
  }

  .card-cover-fallback {
    display: none;
    width: 100%;
    height: 100%;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    padding: 1rem;
    text-align: center;
    background: radial-gradient(circle at 50% 40%, rgba(56, 189, 248, 0.15), transparent 70%);
  }

  .fallback-icon {
    font-size: 2.2rem;
    margin-bottom: 0.5rem;
    opacity: 0.7;
  }

  .fallback-title {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--text);
  }

  .card-hover-overlay {
    position: absolute;
    inset: 0;
    background: rgba(3, 7, 18, 0.4);
    opacity: 0;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: opacity 0.18s ease;
    backdrop-filter: blur(2px);
  }

  .game-card:hover .card-hover-overlay {
    opacity: 1;
  }

  .card-play-btn {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--accent);
    color: #ffffff;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    box-shadow: 0 4px 16px rgba(56, 189, 248, 0.5);
    display: grid;
    place-items: center;
    transition: transform 0.15s ease, background 0.15s ease;
  }

  .card-play-btn:hover {
    transform: scale(1.12);
    background: var(--accent-hover);
  }

  .card-play-btn.install {
    background: var(--ok);
    box-shadow: 0 4px 16px rgba(52, 211, 153, 0.5);
  }

  .card-check {
    position: absolute;
    top: 8px;
    left: 8px;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.4);
    background: rgba(6, 14, 26, 0.65);
    display: grid;
    place-items: center;
    font-size: 0.75rem;
    cursor: pointer;
    z-index: 2;
    transition: all 0.15s ease;
  }

  .card-check.checked {
    border-color: var(--accent);
    background: var(--accent);
    color: #ffffff;
  }

  .card-badges {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-end;
    pointer-events: none;
  }

  .pill-platform {
    font-size: 0.65rem;
    font-weight: 600;
    background: rgba(6, 14, 26, 0.85);
    border: 1px solid var(--border-subtle);
    backdrop-filter: blur(4px);
  }

  .pill-uninstalled {
    font-size: 0.62rem;
    background: rgba(100, 116, 139, 0.7);
    color: #ffffff;
  }

  .card-info {
    padding: 0.65rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .card-title {
    font-weight: 600;
    font-size: 0.85rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .card-fav {
    color: var(--warn);
    font-size: 0.9rem;
  }

  .empty-state {
    grid-column: 1 / -1;
    padding: 3rem;
    text-align: center;
    border-radius: var(--radius);
  }

  /* ========================================================================= */
  /* LIST VIEW STYLES                                                          */
  /* ========================================================================= */
  .list-view {
    padding: 0.45rem;
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-radius: var(--radius);
  }

  .game-list-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    text-align: left;
    padding: 0.55rem 0.75rem;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text);
    cursor: pointer;
    font: inherit;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .game-list-row:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: var(--border-subtle);
  }

  .game-list-row.selected {
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.18), rgba(168, 85, 247, 0.16));
    border-color: var(--border-glow);
  }

  .list-thumb {
    width: 72px;
    height: 34px;
    object-fit: cover;
    border-radius: 4px;
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
    background: rgba(6, 14, 26, 0.5);
  }

  .name {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
    font-weight: 550;
  }

  .tags {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
    justify-content: flex-end;
    max-width: 32%;
  }

  .playtime-col {
    width: 70px;
    text-align: right;
  }

  .size-col {
    width: 72px;
    text-align: right;
  }

  .mini {
    padding: 0.2rem 0.55rem;
    font-size: 0.75rem;
  }

  /* ========================================================================= */
  /* EXTENSIVE GAME DETAILS SUB-PAGE STYLES                                    */
  /* ========================================================================= */
  .game-details-view {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding-bottom: 3rem;
  }

  .details-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.15rem;
    border-radius: var(--radius);
  }

  .game-top-title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 650;
  }

  .play-btn {
    padding: 0.5rem 1.4rem;
    font-size: 0.95rem;
    font-weight: 650;
    letter-spacing: 0.02em;
    background: linear-gradient(135deg, var(--accent), #2563eb);
    box-shadow: 0 4px 18px rgba(56, 189, 248, 0.45);
  }

  .install-btn {
    padding: 0.5rem 1.4rem;
    font-size: 0.95rem;
    font-weight: 650;
    background: linear-gradient(135deg, var(--ok), #059669);
  }

  .fav-btn.is-fav {
    color: var(--warn);
    font-size: 1.2rem;
  }

  .feedback-pill {
    animation: fadeIn 0.2s ease;
  }

  /* Details Hero */
  .details-hero {
    position: relative;
    border-radius: var(--radius-lg);
    overflow: hidden;
    min-height: 260px;
    display: flex;
    align-items: flex-end;
  }

  .hero-backdrop {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center 30%;
    filter: brightness(0.7) blur(1px);
    transform: scale(1.02);
  }

  .hero-overlay {
    position: absolute;
    inset: 0;
    background: linear-gradient(0deg, #030712 5%, rgba(3, 7, 18, 0.6) 50%, rgba(3, 7, 18, 0.2) 100%);
  }

  .hero-content {
    position: relative;
    z-index: 1;
    display: flex;
    gap: 1.5rem;
    padding: 1.5rem 2rem;
    align-items: flex-end;
    width: 100%;
  }

  .hero-capsule {
    width: 130px;
    height: 195px;
    object-fit: cover;
    border-radius: var(--radius-sm);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.7);
    border: 1px solid var(--border);
    flex-shrink: 0;
  }

  .hero-meta {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .hero-title {
    margin: 0;
    font-size: 1.85rem;
    font-weight: 750;
    letter-spacing: -0.02em;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
  }

  .hero-desc {
    margin: 0;
    color: var(--text-dim);
    font-size: 0.9rem;
    max-width: 780px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .hero-tags {
    gap: 0.45rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
  }

  /* Details Grid */
  .details-grid {
    display: grid;
    grid-template-columns: 1.6fr 1fr;
    gap: 1.25rem;
    align-items: flex-start;
  }

  .details-sidebar {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  /* Media Gallery */
  .media-gallery-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .featured-media {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: #000;
    cursor: zoom-in;
    border: 1px solid var(--border-subtle);
  }

  .featured-media img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.25s ease;
  }

  .featured-media:hover img {
    transform: scale(1.02);
  }

  .media-zoom-hint {
    position: absolute;
    bottom: 8px;
    right: 8px;
    background: rgba(0, 0, 0, 0.75);
    color: #fff;
    padding: 0.3rem 0.6rem;
    border-radius: var(--radius-pill);
    font-size: 0.72rem;
    backdrop-filter: blur(4px);
    pointer-events: none;
  }

  .gallery-thumbnails {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
  }

  .thumb-btn {
    border: 2px solid transparent;
    padding: 0;
    background: #000;
    border-radius: 6px;
    overflow: hidden;
    aspect-ratio: 16 / 9;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .thumb-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    opacity: 0.7;
    transition: opacity 0.15s ease;
  }

  .thumb-btn:hover img,
  .thumb-btn.active img {
    opacity: 1;
  }

  .thumb-btn.active {
    border-color: var(--accent);
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.4);
  }

  /* Meta Links List */
  .meta-links-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .meta-link {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-subtle);
    color: var(--text);
    text-decoration: none;
    font-size: 0.85rem;
    transition: all 0.15s ease;
  }

  .meta-link:hover {
    background: var(--panel-hover);
    border-color: var(--border-glow);
    color: var(--accent);
    transform: translateX(3px);
  }

  .meta-icon {
    font-size: 1rem;
  }

  /* Info Table */
  .info-table {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-size: 0.85rem;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.25rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .info-row.full-width {
    flex-direction: column;
    align-items: flex-start;
  }

  .code-val {
    font-family: monospace;
    font-size: 0.75rem;
    background: rgba(0, 0, 0, 0.35);
    padding: 0.2rem 0.45rem;
    border-radius: 4px;
    word-break: break-all;
    color: var(--accent);
  }

  /* Playtime Stats */
  .playtime-stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .stat-box {
    display: flex;
    flex-direction: column;
    background: rgba(0, 0, 0, 0.25);
    padding: 0.85rem;
    border-radius: var(--radius-sm);
    text-align: center;
    border: 1px solid var(--border-subtle);
  }

  .stat-number {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--accent);
  }

  .stat-label {
    font-size: 0.72rem;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-top: 0.25rem;
  }

  /* Modal Sub-Window */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    display: grid;
    place-items: center;
    z-index: 100;
    padding: 1rem;
  }

  .modal {
    width: 100%;
    max-width: 520px;
    border-radius: var(--radius);
    padding: 1.5rem;
    box-shadow: var(--shadow-elevated);
  }

  .modal-head {
    align-items: center;
    margin-bottom: 1.25rem;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 1.15rem;
  }

  .manage-group {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .group-label {
    font-size: 0.78rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-dim);
  }

  .platform-select-modal {
    width: 100%;
    padding: 0.45rem;
    border-radius: var(--radius-sm);
    background: var(--bg-1);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .modal-footer {
    align-items: center;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-subtle);
  }

  /* Lightbox */
  .lightbox-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.92);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    z-index: 200;
    padding: 2rem;
  }

  .lightbox-img {
    max-width: 90vw;
    max-height: 85vh;
    object-fit: contain;
    border-radius: var(--radius);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.8);
    border: 1px solid var(--border-glow);
  }

  .lightbox-close {
    position: absolute;
    top: 24px;
    right: 28px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: #fff;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    font-size: 1.2rem;
    cursor: pointer;
    display: grid;
    place-items: center;
    transition: background 0.15s ease;
  }

  .lightbox-close:hover {
    background: rgba(255, 255, 255, 0.25);
  }
</style>
