<script lang="ts">
  import "$styles/library.scss";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { collections, error, games, loading } from "$lib/stores";
  import type { AppMeta, Game, MediaItem, Platform } from "$lib/types";

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
  let mediaItems: MediaItem[] = [];
  let selectedGalleryIdx = 0;
  let activeLightboxMedia: MediaItem | null = null;
  let showManageModal = false;
  let managingGame: Game | null = null;
  let actionFeedback: string | null = null;
  let customArtworks: Record<number, string> = {};
  let failedAppIds = new Set<number>();

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
      loadLocalArtwork();
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

  function getPlatformIcon(platform: string): string {
    switch (platform) {
      case "steam": return "🎮";
      case "gog": return "🌌";
      case "epic": return "⚡";
      case "itch": return "👾";
      case "ubisoft": return "🌀";
      case "ea": return "⚽";
      case "battlenet": return "⚔️";
      case "xbox": return "🟢";
      case "emulator": return "🕹️";
      default: return "🎲";
    }
  }

  function getGameCover(game: Game): string {
    if (customArtworks[game.appId]) {
      return customArtworks[game.appId];
    }
    if (game.kind === "shortcut" || game.platform !== "steam") {
      if (game.icon && (game.icon.startsWith("http") || game.icon.startsWith("data:") || game.icon.startsWith("asset:") || game.icon.startsWith("file:"))) {
        return game.icon;
      }
      return "";
    }
    return getCapsuleUrl(game.appId);
  }

  function getGameHeader(game: Game): string {
    if (customArtworks[game.appId]) {
      return customArtworks[game.appId];
    }
    if (game.kind === "shortcut" || game.platform !== "steam") {
      if (game.icon && (game.icon.startsWith("http") || game.icon.startsWith("data:") || game.icon.startsWith("asset:") || game.icon.startsWith("file:"))) {
        return game.icon;
      }
      return "";
    }
    return getHeaderUrl(game.appId);
  }

  function handleCoverError(e: Event, game: Game) {
    const img = e.currentTarget as HTMLImageElement;
    const currentSrc = img.src;

    // For non-Steam shortcuts or non-steam platforms, no point querying Steam CDN
    if (game.kind === "shortcut" || game.platform !== "steam") {
      failedAppIds.add(game.appId);
      failedAppIds = new Set(failedAppIds);
      return;
    }

    // 1. If library_600x900.jpg failed for a Steam game, fall back to header.jpg
    if (currentSrc.includes("/library_600x900.jpg")) {
      img.src = `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/header.jpg`;
      return;
    }

    // 2. If header.jpg failed, fall back to capsule_616x353.jpg
    if (currentSrc.includes("/header.jpg")) {
      img.src = `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${game.appId}/capsule_616x353.jpg`;
      return;
    }

    // 3. If game has an icon and we haven't tried it yet
    if (game.icon && !currentSrc.includes(game.icon) && (game.icon.startsWith("http") || game.icon.startsWith("data:"))) {
      img.src = game.icon;
      return;
    }

    // 4. Everything failed: hide image and reveal the styled fallback container
    failedAppIds.add(game.appId);
    failedAppIds = new Set(failedAppIds);
  }

  async function loadLocalArtwork() {
    try {
      const report = await api.getArtwork();
      const withCustomGrid = report.filter((g) =>
        g.slots.some((s) => (s.slot === "gridPortrait" || s.slot === "gridLandscape") && s.present)
      );
      for (const g of withCustomGrid) {
        const hasPortrait = g.slots.some((s) => s.slot === "gridPortrait" && s.present);
        const slot = hasPortrait ? "gridPortrait" : "gridLandscape";
        api.readArtwork(g.appId, slot).then((dataUrl) => {
          if (dataUrl) {
            customArtworks[g.appId] = dataUrl;
            customArtworks = { ...customArtworks };
          }
        }).catch(() => {});
      }
    } catch (e) {
      console.debug("Could not inspect local custom grid artwork:", e);
    }
  }

  function buildMediaGallery(game: Game, meta?: AppMeta | null): MediaItem[] {
    const items: MediaItem[] = [];
    const seenUrls = new Set<string>();

    // 1. Up to 2 videos first (game trailers / gameplay previews)
    if (meta?.movies && Array.isArray(meta.movies)) {
      for (const movieUrl of meta.movies) {
        if (items.length >= 2) break;
        if (!movieUrl || seenUrls.has(movieUrl)) continue;
        seenUrls.add(movieUrl);
        items.push({
          type: "video",
          url: movieUrl,
          title: "Trailer / Preview"
        });
      }
    }

    // 2. Real screenshots from metadata
    if (meta?.screenshots && Array.isArray(meta.screenshots)) {
      for (const ssUrl of meta.screenshots) {
        if (items.length >= 10) break;
        if (!ssUrl || seenUrls.has(ssUrl)) continue;
        seenUrls.add(ssUrl);
        items.push({
          type: "image",
          url: ssUrl,
          title: "Screenshot"
        });
      }
    }

    // 3. Official Store Artwork (hero, header, capsule, logo) if not already added, capped at 10 items
    const artworkCandidates: (string | undefined)[] = [];
    if (meta?.heroImage) artworkCandidates.push(meta.heroImage);
    if (game.kind === "steam" || game.platform === "steam" || game.appId < 0x80000000) {
      artworkCandidates.push(getHeroUrl(game.appId));
    }
    if (meta?.headerImage) artworkCandidates.push(meta.headerImage);
    if (game.kind === "steam" || game.platform === "steam" || game.appId < 0x80000000) {
      artworkCandidates.push(getHeaderUrl(game.appId));
    }
    if (meta?.capsuleImage) artworkCandidates.push(meta.capsuleImage);
    if (game.kind === "steam" || game.platform === "steam" || game.appId < 0x80000000) {
      artworkCandidates.push(getCapsuleUrl(game.appId));
    }
    if (meta?.logoImage) artworkCandidates.push(meta.logoImage);
    if (game.kind === "steam" || game.platform === "steam" || game.appId < 0x80000000) {
      artworkCandidates.push(getLogoUrl(game.appId));
    }

    for (const art of artworkCandidates) {
      if (items.length >= 10) break;
      if (!art || seenUrls.has(art)) continue;
      seenUrls.add(art);
      items.push({
        type: "image",
        url: art,
        title: "Artwork"
      });
    }

    // Strict cap at 10 items without artificial padding or duplicates
    return items.slice(0, 10);
  }

  function handleMediaError(idx: number) {
    if (idx >= 0 && idx < mediaItems.length) {
      mediaItems = mediaItems.filter((_, i) => i !== idx);
      if (selectedGalleryIdx >= mediaItems.length) {
        selectedGalleryIdx = Math.max(0, mediaItems.length - 1);
      }
    }
  }

  // Open extensive per-game details view
  async function openGameDetails(game: Game) {
    selectedGame = game;
    gameMeta = null;
    loadingMeta = true;
    selectedGalleryIdx = 0;
    actionFeedback = null;
    mediaItems = buildMediaGallery(game, null);

    try {
      if (game.kind === "steam" || game.appId < 0x80000000) {
        const metas = await api.fetchSteamStoreMetadata([game.appId]);
        if (metas.length > 0) {
          gameMeta = metas[0];
          mediaItems = buildMediaGallery(game, gameMeta);
        }
      }
      if (!gameMeta && game.name) {
        const catalogMeta = await api.fetchOnlineGameEntry(game.name).catch(() => null);
        if (catalogMeta) {
          gameMeta = catalogMeta;
          mediaItems = buildMediaGallery(game, gameMeta);
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
    managingGame = null;
    actionFeedback = null;
    mediaItems = [];
    activeLightboxMedia = null;
  }

  function openManageModal(game: Game, e?: Event) {
    if (e) e.stopPropagation();
    managingGame = game;
    showManageModal = true;
  }

  function closeManageModal() {
    showManageModal = false;
    managingGame = null;
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
  {@const detailCover = getGameCover(selectedGame)}
  <div class="game-details-view">
    <!-- Sub-page Header bar -->
    <div class="details-topbar glass-strong">
      <div class="row topbar-nav-group">
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

      <div class="row topbar-action-group">
        {#if actionFeedback}
          <span class="pill pill-glow feedback-pill">{actionFeedback}</span>
        {/if}

        {#if selectedGame.installed}
          <button class="btn btn-primary play-btn" onclick={() => handleLaunch(selectedGame!)}>
            <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor" class="btn-glyph"><path d="M4.5 3.2v9.6l8-4.8z" /></svg>
            Play
          </button>
        {:else}
          <button class="btn btn-primary install-btn" onclick={() => handleInstall(selectedGame!)}>
            <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="btn-glyph"><path d="M8 2v8m0 0l-3.2-3.2m3.2 3.2l3.2-3.2" /><path d="M2.5 13.5h11" /></svg>
            Install
          </button>
        {/if}

        <button class="btn btn-ghost" onclick={() => openManageModal(selectedGame!)} title="Manage Game Options">
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
        style="--hero-bg: url('{getHeroUrl(selectedGame.appId)}'); --header-bg: url('{getHeaderUrl(selectedGame.appId)}');"
      ></div>
      <div class="hero-overlay"></div>
      <div class="hero-content">
        {#if detailCover && !failedAppIds.has(selectedGame.appId)}
          <img
            class="hero-capsule"
            src={detailCover}
            alt={selectedGame.name}
            onerror={(e) => handleCoverError(e, selectedGame!)}
          />
        {:else}
          <div class="hero-capsule hero-capsule-fallback platform-{selectedGame.platform}">
            <span class="fallback-icon">{getPlatformIcon(selectedGame.platform)}</span>
            <span class="fallback-platform">{platformLabels[selectedGame.platform] || selectedGame.platform}</span>
          </div>
        {/if}
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
      <!-- Media Gallery (Videos & Images, Capped at 10) -->
      <section class="glass-strong card media-gallery-card">
        <div class="row-between section-head">
          <div>
            <h3 class="section-title">Media Gallery</h3>
            <p class="muted-sm">Up to 10 trailers, gameplay previews, screenshots, and artwork. Click to expand.</p>
          </div>
          {#if mediaItems.length > 0}
            <span class="muted-sm">{selectedGalleryIdx + 1} / {mediaItems.length}</span>
          {/if}
        </div>

        {#if mediaItems.length > 0 && mediaItems[selectedGalleryIdx]}
          {@const currentMedia = mediaItems[selectedGalleryIdx]}
          <!-- Featured Media Display -->
          <div class="featured-media" class:video-mode={currentMedia.type === "video"}>
            {#if currentMedia.type === "video"}
              <video
                src={currentMedia.url}
                controls
                autoplay
                muted
                loop
                playsinline
                class="featured-video-player"
                onerror={() => handleMediaError(selectedGalleryIdx)}
              >
                <track kind="captions" />
              </video>
              <button
                class="media-zoom-hint"
                onclick={() => (activeLightboxMedia = currentMedia)}
                title="Expand video"
              >
                🔍 Fullscreen
              </button>
            {:else}
              <div
                class="featured-image-wrapper"
                role="button"
                tabindex="0"
                onclick={() => (activeLightboxMedia = currentMedia)}
                onkeydown={(e) => e.key === "Enter" && (activeLightboxMedia = currentMedia)}
              >
                <img
                  src={currentMedia.url}
                  alt={currentMedia.title || "Game media"}
                  onerror={() => handleMediaError(selectedGalleryIdx)}
                />
                <div class="media-zoom-hint">🔍 Expand to Fullscreen</div>
              </div>
            {/if}
          </div>

          <!-- Thumbnail Strip (Capped at 10 items) -->
          <div class="gallery-thumbnails">
            {#each mediaItems as item, idx}
              <button
                class="thumb-btn"
                class:active={selectedGalleryIdx === idx}
                class:is-video={item.type === "video"}
                onclick={() => (selectedGalleryIdx = idx)}
                title={item.type === "video" ? `Video: ${item.title || "Trailer"}` : (item.title || `Image ${idx + 1}`)}
              >
                {#if item.type === "video"}
                  <div class="thumb-video-wrapper">
                    <video
                      src={item.url}
                      preload="metadata"
                      muted
                      playsinline
                      onerror={() => handleMediaError(idx)}
                    ></video>
                    <span class="video-play-badge" aria-hidden="true">▶</span>
                  </div>
                {:else}
                  <img
                    src={item.url}
                    alt={item.title || `Thumbnail ${idx + 1}`}
                    onerror={() => handleMediaError(idx)}
                  />
                {/if}
              </button>
            {/each}
          </div>
        {:else}
          <div class="no-media-placeholder">
            <span class="muted">No media preview available for this title.</span>
          </div>
        {/if}
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
        <p class="muted loading-notice">{$t("common.loading")}</p>
      {:else if viewMode === "grid"}
        <!-- GRID DISPLAY (Default & Primary Layout) -->
        <div class="games-grid">
          {#each filtered as game (game.kind + "-" + game.appId)}
            {@const coverSrc = getGameCover(game)}
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
              <div class="card-cover-container" class:has-fallback={!coverSrc || failedAppIds.has(game.appId)}>
                {#if coverSrc && !failedAppIds.has(game.appId)}
                  <img
                    class="card-cover"
                    src={coverSrc}
                    alt={game.name}
                    loading="lazy"
                    onerror={(e) => handleCoverError(e, game)}
                  />
                {/if}
                <div class="card-cover-fallback platform-{game.platform}">
                  <div class="fallback-glow"></div>
                  <div class="fallback-content">
                    <span class="fallback-icon">{getPlatformIcon(game.platform)}</span>
                    <span class="fallback-title">{game.name || `App ${game.appId}`}</span>
                    <span class="fallback-platform">{platformLabels[game.platform] || game.platform}</span>
                  </div>
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
                  <div class="card-meta-left">
                    <span class="muted-sm">{formatPlaytime(game.playtimeMinutes)}</span>
                    {#if game.favorite}<span class="card-fav">★</span>{/if}
                  </div>
                  <!-- Bottom right actions: Play / Download -->
                  <!-- svelte-ignore a11y_click_events_have_key_events -->
                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                  <div class="card-actions" onclick={(e) => e.stopPropagation()}>
                    {#if game.installed}
                      <button
                        class="card-action-btn play"
                        title="Launch {game.name}"
                        onclick={(e) => {
                          e.stopPropagation();
                          handleLaunch(game);
                        }}
                      >
                        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor" class="action-glyph">
                          <path d="M4.5 3.2v9.6l8-4.8z" />
                        </svg>
                      </button>
                    {:else}
                      <button
                        class="card-action-btn install"
                        title="Install {game.name}"
                        onclick={(e) => {
                          e.stopPropagation();
                          handleInstall(game);
                        }}
                      >
                        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="action-glyph">
                          <path d="M8 2v8m0 0l-3.2-3.2m3.2 3.2l3.2-3.2" />
                          <path d="M2.5 13.5h11" />
                        </svg>
                      </button>
                    {/if}
                  </div>
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
            {@const thumbSrc = getGameHeader(game)}
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

              {#if thumbSrc}
                <img
                  src={thumbSrc}
                  alt=""
                  class="list-thumb"
                  onerror={(e) => ((e.currentTarget as HTMLElement).style.display = 'none')}
                />
              {:else}
                <div class="list-thumb list-thumb-fallback">
                  <span>{getPlatformIcon(game.platform)}</span>
                </div>
              {/if}

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
              <div class="row list-actions-group" onclick={(e) => e.stopPropagation()}>
                {#if game.installed}
                  <button class="btn mini btn-primary" onclick={() => handleLaunch(game)}>
                    <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor" class="action-glyph-inline"><path d="M4.5 3.2v9.6l8-4.8z" /></svg>
                    Play
                  </button>
                {:else}
                  <button class="btn mini" onclick={() => handleInstall(game)}>
                    <svg viewBox="0 0 16 16" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="action-glyph-inline"><path d="M8 2v8m0 0l-3.2-3.2m3.2 3.2l3.2-3.2" /><path d="M2.5 13.5h11" /></svg>
                    Install
                  </button>
                {/if}
              </div>
            </div>
          {/each}
          {#if filtered.length === 0}
            <p class="muted empty-list-notice">No games match the current filters.</p>
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

<!-- Management Sub-Window Modal (Accessible from both Grid Cards & Details View) -->
{#if showManageModal && (managingGame || selectedGame)}
  {@const targetGame = managingGame || selectedGame!}
  <div
    class="modal-backdrop"
    role="button"
    tabindex="0"
    onclick={closeManageModal}
    onkeydown={(e) => e.key === "Escape" && closeManageModal()}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal glass-strong card" onclick={(e) => e.stopPropagation()}>
      <div class="row-between modal-head">
        <h2 class="section-title">Manage: {targetGame.name}</h2>
        <button class="btn btn-ghost" onclick={closeManageModal}>✕</button>
      </div>

      <div class="modal-body">
        <!-- Folder Management -->
        <div class="manage-group">
          <span class="group-label">Local Files & Installation</span>
          <div class="row" style="gap:0.5rem">
            <button
              class="btn"
              onclick={() => handleOpenFolder(targetGame)}
              disabled={!targetGame.installDir && !targetGame.startDir}
            >
              📂 Open Game Folder
            </button>
            {#if targetGame.installed}
              <button class="btn btn-danger" onclick={() => handleUninstall(targetGame)}>
                🗑️ Uninstall Game
              </button>
            {:else}
              <button class="btn btn-primary" onclick={() => handleInstall(targetGame)}>
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
            value={targetGame.platform}
            onchange={(e) => setPlatform(targetGame, e.currentTarget.value)}
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
                selected = new Set([targetGame.appId]);
                assign("add");
              }}
            >
              Add
            </button>
            <button
              class="btn btn-danger"
              onclick={() => {
                selected = new Set([targetGame.appId]);
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
              <span class="code-val">{targetGame.launchUri || `steam://run/${targetGame.appId}`}</span>
            </div>
            {#if targetGame.installUri}
              <div class="info-row full-width">
                <span class="muted">Install URI:</span>
                <span class="code-val">{targetGame.installUri}</span>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <div class="modal-footer row-between">
        <span class="muted-sm">Changes take effect immediately.</span>
        <button class="btn btn-primary" onclick={closeManageModal}>Done</button>
      </div>
    </div>
  </div>
{/if}

<!-- Fullscreen Lightbox Modal -->
{#if activeLightboxMedia}
  <div
    class="lightbox-overlay"
    role="button"
    tabindex="0"
    onclick={() => (activeLightboxMedia = null)}
    onkeydown={(e) => e.key === "Escape" && (activeLightboxMedia = null)}
  >
    {#if activeLightboxMedia.type === "video"}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="lightbox-video-container" onclick={(e) => e.stopPropagation()}>
        <video
          src={activeLightboxMedia.url}
          controls
          autoplay
          playsinline
          class="lightbox-video"
        >
          <track kind="captions" />
        </video>
      </div>
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <img
        src={activeLightboxMedia.url}
        alt={activeLightboxMedia.title || "Enlarged media"}
        class="lightbox-img"
        onclick={(e) => e.stopPropagation()}
      />
    {/if}
    <button class="lightbox-close" onclick={() => (activeLightboxMedia = null)}>✕</button>
  </div>
{/if}

