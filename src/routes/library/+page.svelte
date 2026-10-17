<script lang="ts">
  import "$styles/library.scss";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t, locale, activeLanguage } from "$lib/i18n";
  import { customNames, error, games, loading, selectedGameId, settings } from "$lib/stores";
  import { getPlatformMetadata, pluginEngine } from "$lib/plugins";
  import AddGameModal from "$lib/components/library/AddGameModal.svelte";
  import type { AppMeta, Game, MediaItem } from "$lib/types";

  // Per-game details state
  let selectedGame = $state<Game | null>(null);
  let gameMeta = $state<AppMeta | null>(null);
  let loadingMeta = $state(false);
  let mediaItems = $state<MediaItem[]>([]);
  let selectedGalleryIdx = $state(0);
  let activeLightboxMedia = $state<MediaItem | null>(null);
  let showManageModal = $state(false);
  let showAddGameModal = $state(false);
  let isStandaloneEnabled = $state(true);
  let managingGame = $state<Game | null>(null);
  let actionFeedback = $state<string | null>(null);
  let modalCustomName = $state("");
  let modalFeedback = $state<string | null>(null);
  let selectedEnhanceProvider = $state("all");
  let overrideExisting = $state(false);
  let isEnhancing = $state(false);

  function getDisplayName(game: Game | null | undefined): string {
    if (!game) return "";
    return $customNames[game.appId] || game.name || `App ${game.appId}`;
  }

  function getLocalizedTitle(game: Game | null | undefined, meta?: AppMeta | null): string {
    if (!game) return "";
    if ($customNames[game.appId]) return $customNames[game.appId];
    if (meta?.locales) {
      const lang = $locale.toLowerCase();
      const steamLang = $activeLanguage.steamCode.toLowerCase();
      const match = meta.locales[lang] || meta.locales[steamLang] || meta.locales[lang.split("-")[0]];
      if (match?.title) return match.title;
    }
    return game.name || `App ${game.appId}`;
  }

  function getLocalizedDescription(meta?: AppMeta | null): string {
    if (!meta) return "";
    if (meta.locales) {
      const lang = $locale.toLowerCase();
      const steamLang = $activeLanguage.steamCode.toLowerCase();
      const match = meta.locales[lang] || meta.locales[steamLang] || meta.locales[lang.split("-")[0]];
      if (match?.shortDescription) return match.shortDescription;
    }
    return meta.shortDescription || "";
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

  function getGameCover(game: Game): string {
    if (game.capsuleImage) return game.capsuleImage;
    if (game.platform === "steam") return getCapsuleUrl(game.appId);
    if (game.icon && (game.icon.startsWith("http") || game.icon.startsWith("data:") || game.icon.startsWith("asset:"))) {
      return game.icon;
    }
    return "";
  }

  function getGameHero(game: Game): string {
    if (game.heroImage) return game.heroImage;
    if (game.platform === "steam") return getHeroUrl(game.appId);
    if (game.capsuleImage) return game.capsuleImage;
    return "";
  }

  function buildMediaGallery(game: Game, meta?: AppMeta | null): MediaItem[] {
    const items: MediaItem[] = [];
    const seenUrls = new Set<string>();

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

    const artworkCandidates: (string | undefined)[] = [];
    if (game.heroImage) artworkCandidates.push(game.heroImage);
    if (meta?.heroImage) artworkCandidates.push(meta.heroImage);
    if (game.platform === "steam") artworkCandidates.push(getHeroUrl(game.appId));
    if (game.capsuleImage) artworkCandidates.push(game.capsuleImage);
    if (meta?.capsuleImage) artworkCandidates.push(meta.capsuleImage);
    if (game.platform === "steam") artworkCandidates.push(getCapsuleUrl(game.appId));

    for (const art of artworkCandidates) {
      if (items.length >= 10) break;
      if (!art || seenUrls.has(art)) continue;
      seenUrls.add(art);
      items.push({
        type: "image",
        url: art,
        title: "Official Artwork"
      });
    }

    return items;
  }

  async function loadGameDetails(game: Game) {
    selectedGame = game;
    selectedGalleryIdx = 0;
    loadingMeta = true;

    try {
      // 1. Check persistent KV cache first
      let meta = await api.kvGetJson<AppMeta>(`game_meta:${game.appId}`);
      if (!meta) {
        meta = await api.getCachedGameMetadata(game.appId, game.name, game.exe);
      }

      // 2. Default source per platform is the game provider:
      // If no metadata or missing provider fields, query the native provider plugin
      if (!meta || (!meta.shortDescription && !meta.developers?.length && !meta.heroImage)) {
        const providerData = await pluginEngine.fetchProviderGameData(game);
        if (providerData) {
          meta = pluginEngine.mergeProviderData(meta, providerData, game, false);
          await api.kvSetJson(`game_meta:${game.appId}`, meta);
        }
      }

      gameMeta = meta;
      mediaItems = buildMediaGallery(game, meta);
    } catch {
      gameMeta = null;
      mediaItems = buildMediaGallery(game, null);
    } finally {
      loadingMeta = false;
    }
  }

  async function refreshCurrentGameMetadata(game: Game, isReset = false) {
    loadingMeta = true;
    try {
      let meta = gameMeta;
      // 1. Fetch from native platform provider plugin
      const providerData = await pluginEngine.fetchProviderGameData(game);

      // 2. If it's a native Steam game, also query the backend Steam refresh
      if (game.platform === "steam" || game.kind === "steam") {
        try {
          const steamMeta = await api.refreshGameMetadata(game.appId, isReset);
          if (steamMeta) {
            meta = steamMeta;
          }
        } catch {
          // Non-fatal if backend steam call fails
        }
      }

      if (providerData) {
        meta = pluginEngine.mergeProviderData(meta, providerData, game, isReset);
      }

      if (meta) {
        await api.kvSetJson(`game_meta:${game.appId}`, meta);
      }

      gameMeta = meta;
      mediaItems = buildMediaGallery(game, meta);
      actionFeedback = isReset
        ? "Metadata cache reset to provider defaults!"
        : "Metadata refreshed from provider source!";
      setTimeout(() => (actionFeedback = null), 4000);
    } catch (e) {
      actionFeedback = `Metadata refresh failed: ${errorMessage(e)}`;
    } finally {
      loadingMeta = false;
    }
  }

  async function handleEnhanceMetadata(
    game: Game,
    options?: { providerId?: string; override?: boolean }
  ) {
    isEnhancing = true;
    try {
      const targetProvider = options?.providerId && options.providerId !== "all"
        ? options.providerId
        : (selectedEnhanceProvider !== "all" ? selectedEnhanceProvider : undefined);

      const { meta, enhancedBy } = await pluginEngine.enhanceGameMetadata(gameMeta, game, {
        providerId: targetProvider,
        override: options?.override ?? overrideExisting
      });

      gameMeta = meta;
      mediaItems = buildMediaGallery(game, meta);
      const provList = enhancedBy.length ? enhancedBy.join(", ") : "Metadata providers";
      modalFeedback = `Enhanced details using: ${provList}`;
      actionFeedback = `Enhanced metadata via ${provList}!`;
      setTimeout(() => {
        actionFeedback = null;
        modalFeedback = null;
      }, 5000);
    } catch (e) {
      const err = `Enhancement failed: ${errorMessage(e)}`;
      modalFeedback = err;
      actionFeedback = err;
    } finally {
      isEnhancing = false;
    }
  }

  async function handleResetMetadataCache(game: Game) {
    loadingMeta = true;
    try {
      await api.kvDelete(`game_meta:${game.appId}`);
      await refreshCurrentGameMetadata(game, true);
      modalFeedback = "Metadata cache reset to provider defaults.";
      setTimeout(() => (modalFeedback = null), 4000);
    } catch (e) {
      modalFeedback = `Reset failed: ${errorMessage(e)}`;
    } finally {
      loadingMeta = false;
    }
  }

  function getFullInstallPath(game: Game | null | undefined): string | null {
    if (!game) return null;
    if (game.installDir) {
      if (game.installDir.includes(":") || game.installDir.startsWith("/") || game.installDir.startsWith("\\")) {
        return game.installDir;
      }
      if (game.libraryPath) {
        const cleanLib = game.libraryPath.replace(/[\/\\]+$/, "");
        return `${cleanLib}\\steamapps\\common\\${game.installDir}`;
      }
      return game.installDir;
    }
    if (game.exe) {
      const lastSlash = Math.max(game.exe.lastIndexOf("/"), game.exe.lastIndexOf("\\"));
      if (lastSlash > 0) return game.exe.slice(0, lastSlash);
      return game.exe;
    }
    return game.libraryPath || null;
  }

  async function handleLaunch(game: Game) {
    actionFeedback = "Launching game…";
    try {
      await api.launchGame(game.appId, game.launchUri, game.exe, game.platform);
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

  async function handleOpenFolder(game: Game) {
    const path = getFullInstallPath(game);
    if (!path) {
      actionFeedback = "No install folder known for this game.";
      return;
    }
    try {
      await api.openGameFolder(path);
    } catch (e) {
      actionFeedback = `Could not open folder: ${errorMessage(e)}`;
    }
  }

  function toggleFavorite(game: Game) {
    const newFav = !game.favorite;
    game.favorite = newFav;
    games.update((list) => list.map((g) => (g.appId === game.appId ? { ...g, favorite: newFav } : g)));
  }

  function openManageModal(game: Game) {
    managingGame = game;
    modalCustomName = $customNames[game.appId] ?? game.name;
    modalFeedback = null;
    showManageModal = true;
  }

  function saveCustomName() {
    if (!managingGame) return;
    const clean = modalCustomName.trim();
    customNames.update((names) => {
      const copy = { ...names };
      if (!clean || clean === managingGame?.name) {
        delete copy[managingGame!.appId];
      } else {
        copy[managingGame!.appId] = clean;
      }
      return copy;
    });
    modalFeedback = "Saved display name.";
    setTimeout(() => {
      showManageModal = false;
      managingGame = null;
      modalFeedback = null;
    }, 800);
  }

  // Reactive synchronizer: When $selectedGameId changes or games are populated
  $effect(() => {
    const id = $selectedGameId;
    const all = $games;
    if (all.length > 0) {
      const target = all.find((g) => g.appId === id) || all[0];
      if (target && target.appId !== selectedGame?.appId) {
        loadGameDetails(target);
      }
    } else {
      selectedGame = null;
    }
  });

  onMount(async () => {
    try {
      const plugins = await api.getInstalledPlugins();
      const standalone = plugins.find((p) => p.id === "platform-standalone");
      isStandaloneEnabled = standalone ? standalone.enabled : true;
    } catch {
      isStandaloneEnabled = true;
    }

    if ($games.length === 0) {
      loading.set(true);
      try {
        const lib = await api.loadLibrary();
        games.set(lib);
        if (!$selectedGameId && lib.length > 0) {
          selectedGameId.set(lib[0].appId);
        }
      } catch (e) {
        error.set(errorMessage(e));
      } finally {
        loading.set(false);
      }
    }
  });
</script>

<div class="library-container">
  {#if selectedGame}
    {@const coverUrl = getGameCover(selectedGame)}
    {@const heroUrl = getGameHero(selectedGame)}

    <!-- Top Action Bar -->
    <div class="details-topbar glass">
      <div class="topbar-title-block">
        <h2 class="topbar-game-name">{getLocalizedTitle(selectedGame, gameMeta)}</h2>
        <span class="pill pill-accent2">{selectedGame.console || getPlatformMetadata(selectedGame.platform).label}</span>
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
            <svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor" class="btn-glyph">
              <path d="M4.5 3.2v9.6l8-4.8z" />
            </svg>
            Play
          </button>
        {:else}
          <button class="btn btn-primary install-btn" onclick={() => handleInstall(selectedGame!)}>
            <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" class="btn-glyph">
              <path d="M8 2v8m0 0l-3.2-3.2m3.2 3.2l3.2-3.2" />
              <path d="M2.5 13.5h11" />
            </svg>
            Install
          </button>
        {/if}

        <button
          class="btn btn-ghost"
          onclick={() => refreshCurrentGameMetadata(selectedGame!)}
          title={$t("library.refreshMetadata")}
          disabled={loadingMeta}
        >
          <span class:spin={loadingMeta}>🔄</span>
          <span>{loadingMeta ? $t("library.refreshingMetadata") : $t("library.refreshMetadata")}</span>
        </button>

        <button
          class="btn btn-ghost"
          onclick={() => handleEnhanceMetadata(selectedGame!)}
          title="Enhance game details and fill in missing metadata from metadata sources"
          disabled={loadingMeta || isEnhancing}
        >
          <span class:spin={isEnhancing}>✨</span>
          <span>{isEnhancing ? "Enhancing…" : "Enhance Metadata"}</span>
        </button>

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

        {#if isStandaloneEnabled}
          <button
            class="btn btn-ghost add-game-btn"
            onclick={() => (showAddGameModal = true)}
            title="Add local game manually (Standalone & Local Executables)"
          >
            ➕ Add Game
          </button>
        {/if}
      </div>
    </div>

    <!-- Hero Banner -->
    <div class="details-hero glass">
      <div
        class="hero-backdrop"
        style={heroUrl ? `--hero-bg: url('${heroUrl}');` : ""}
      ></div>
      <div class="hero-overlay"></div>
      <div class="hero-content">
        {#if coverUrl}
          <img class="hero-capsule" src={coverUrl} alt={selectedGame.name} />
        {:else}
          <div class="hero-capsule hero-capsule-fallback platform-{selectedGame.platform}">
            <span class="fallback-icon">{getPlatformMetadata(selectedGame.platform).icon}</span>
            <span class="fallback-platform">{getPlatformMetadata(selectedGame.platform).label}</span>
          </div>
        {/if}

        <div class="hero-meta">
          <h1 class="hero-title">{getLocalizedTitle(selectedGame, gameMeta)}</h1>
          {#if getLocalizedDescription(gameMeta)}
            <p class="hero-desc">{getLocalizedDescription(gameMeta)}</p>
          {/if}
          <div class="row hero-tags">
            <span class="pill pill-accent">{getPlatformMetadata(selectedGame.platform).label}</span>
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
      <!-- Media Gallery -->
      <section class="glass-strong card media-gallery-card">
        <div class="row-between section-head">
          <div>
            <h3 class="section-title">Media Gallery</h3>
            <p class="muted-sm">Up to 10 high-resolution screenshots and official artwork. Click to expand.</p>
          </div>
          {#if mediaItems.length > 0}
            <span class="muted-sm">{selectedGalleryIdx + 1} / {mediaItems.length}</span>
          {/if}
        </div>

        {#if mediaItems.length > 0 && mediaItems[selectedGalleryIdx]}
          {@const currentMedia = mediaItems[selectedGalleryIdx]}
          <div class="featured-media">
            <div
              class="featured-image-wrapper"
              role="button"
              tabindex="0"
              onclick={() => (activeLightboxMedia = currentMedia)}
              onkeydown={(e) => e.key === "Enter" && (activeLightboxMedia = currentMedia)}
            >
              <img src={currentMedia.url} alt={currentMedia.title || "Game media"} />
              <div class="media-zoom-hint">🔍 Expand to Fullscreen</div>
            </div>
          </div>

          <div class="gallery-thumbnails">
            {#each mediaItems as item, idx}
              <button
                class="thumb-btn"
                class:active={selectedGalleryIdx === idx}
                onclick={() => (selectedGalleryIdx = idx)}
                title={item.title || `Image ${idx + 1}`}
              >
                <img src={item.url} alt={item.title || `Thumbnail ${idx + 1}`} />
              </button>
            {/each}
          </div>
        {:else}
          <div class="no-media-placeholder">
            <span class="muted">No media preview available for this title.</span>
          </div>
        {/if}
      </section>

      <!-- Right Column: Info Cards -->
      <div class="details-sidebar">
        <!-- Meta Links Card -->
        <div class="glass-strong card">
          <h3 class="section-title">Meta &amp; Reference Links</h3>
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
              <span class="meta-icon">🔧</span> PCGamingWiki Fixes &amp; Settings
            </a>
            <a
              class="meta-link"
              href={`https://en.wikipedia.org/wiki/Special:Search?search=${encodeURIComponent(selectedGame.name)}`}
              target="_blank"
              rel="noreferrer"
            >
              <span class="meta-icon">📖</span> Wikipedia Article
            </a>
          </div>
        </div>

        <!-- Details Card -->
        <div class="glass-strong card">
          <h3 class="section-title">Game Information</h3>
          <div class="info-table">
            <div class="info-row">
              <span class="muted">Platform:</span>
              <span>{selectedGame.console || getPlatformMetadata(selectedGame.platform).label}</span>
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

        <!-- Installation & Storage Card -->
        <div class="glass-strong card">
          <div class="row-between" style="align-items:center">
            <h3 class="section-title" style="margin:0">Installation &amp; Storage</h3>
            <span class="pill" class:pill-ok={selectedGame.installed} class:pill-dim={!selectedGame.installed}>
              {selectedGame.installed ? "Installed" : "Cloud"}
            </span>
          </div>

          <div class="info-table" style="margin-top:0.4rem">
            <div class="info-row">
              <span class="muted">Disk Footprint:</span>
              <span style="font-weight:600;color:var(--accent)">{formatSize(selectedGame.sizeOnDisk)}</span>
            </div>
            {#if getFullInstallPath(selectedGame)}
              <div class="info-row full-width">
                <span class="muted">Install Folder:</span>
                <span class="code-val" title={getFullInstallPath(selectedGame)}>{getFullInstallPath(selectedGame)}</span>
              </div>
            {/if}
            {#if selectedGame.exe}
              <div class="info-row full-width">
                <span class="muted">Executable:</span>
                <span class="code-val" title={selectedGame.exe}>{selectedGame.exe}</span>
              </div>
            {/if}
            {#if selectedGame.libraryPath}
              <div class="info-row full-width">
                <span class="muted">Library Host:</span>
                <span class="code-val">{selectedGame.libraryPath}</span>
              </div>
            {/if}
          </div>

          <div class="row" style="gap:0.5rem;margin-top:0.6rem;flex-wrap:wrap">
            {#if getFullInstallPath(selectedGame)}
              <button
                type="button"
                class="btn mini btn-ghost"
                onclick={() => handleOpenFolder(selectedGame!)}
                title="Open game directory in file explorer"
              >
                📂 Open Folder
              </button>
            {/if}
          </div>
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
  {:else}
    <div class="empty-library-state glass">
      <div class="empty-icon">🎮</div>
      <h2>No Games in Library</h2>
      <p class="muted">Scan your installed gaming platforms or connect accounts in Platforms to populate your unified library.</p>
      {#if isStandaloneEnabled}
        <button
          type="button"
          class="btn btn-primary"
          style="margin-top: 1.25rem;"
          onclick={() => (showAddGameModal = true)}
        >
          ➕ Add Local Game Manually
        </button>
      {/if}
    </div>
  {/if}
</div>

<!-- Manage Game Modal (Display Name & Launch Options) -->
{#if showManageModal && managingGame}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal-backdrop" onclick={() => (showManageModal = false)} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="modal-dialog glass-strong" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="row-between modal-header">
        <div class="modal-title-group">
          <h2>Manage Game</h2>
          <span class="muted-sm">{managingGame.name} (App ID: {managingGame.appId})</span>
        </div>
        <button class="btn btn-ghost close-btn" onclick={() => (showManageModal = false)}>✕</button>
      </div>

      {#if modalFeedback}
        <div class="pill pill-accent" style="margin-bottom:1rem;display:block;text-align:center">
          {modalFeedback}
        </div>
      {/if}

      <div class="modal-body">
        <label class="form-group">
          <span class="form-label">Local Display Name:</span>
          <input
            type="text"
            class="input-text"
            bind:value={modalCustomName}
            placeholder={managingGame.name}
          />
          <span class="muted-xs">Customizes the title shown in the sidebar and showcase.</span>
        </label>

        <div class="form-group" style="margin-top:1rem">
          <span class="form-label">Launch Details:</span>
          <div class="code-val" style="word-break:break-all">
            {managingGame.launchUri || managingGame.exe || "Platform default"}
          </div>
        </div>

        <div class="form-group" style="margin-top:1.25rem;padding-top:1rem;border-top:1px solid rgba(255,255,255,0.08)">
          <span class="form-label">Metadata & Provider Management</span>
          
          <div style="display:flex;align-items:center;justify-content:space-between;margin-top:0.5rem;font-size:0.85rem">
            <span class="muted">Default Game Provider:</span>
            <span class="pill pill-default">
              {getPlatformMetadata(managingGame.platform).icon} {getPlatformMetadata(managingGame.platform).label}
            </span>
          </div>

          <div style="display:flex;align-items:center;justify-content:space-between;margin-top:0.5rem;font-size:0.85rem">
            <span class="muted">Enhancement Status:</span>
            {#if gameMeta?.userEnhanced}
              <span class="pill pill-accent" title="Enhanced from metadata providers">
                ✨ Enhanced ({gameMeta.enhancedBy?.join(", ") || "Metadata Sources"})
              </span>
            {:else}
              <span class="pill pill-default">Default Provider Data</span>
            {/if}
          </div>

          <div style="margin-top:1rem">
            <label for="manual-enhance-source" class="form-label" style="font-size:0.8rem">Manual Metadata Enhancement Source:</label>
            <select id="manual-enhance-source" class="input-text" bind:value={selectedEnhanceProvider} style="width:100%;margin-top:0.25rem">
              <option value="all">All Enabled Metadata Sources (Sequential Fallback)</option>
              <option value="metadata-screenscraper">ScreenScraper (Arcade & Retro Consoles)</option>
              <option value="metadata-thegamesdb">TheGamesDB (Community Game Database)</option>
              <option value="metadata-igdb">IGDB (Twitch / Amazon Games Database)</option>
              <option value="metadata-steamgriddb">SteamGridDB (High-Res Covers & Grids)</option>
              <option value="metadata-hltb">HowLongToBeat (Playtime Statistics)</option>
              <option value="metadata-pcgamingwiki">PCGamingWiki (Configuration & Fixes)</option>
              <option value="metadata-protondb">ProtonDB (Linux & Deck Compatibility)</option>
            </select>
          </div>

          <label style="display:flex;align-items:center;gap:0.5rem;margin-top:0.75rem;cursor:pointer;font-size:0.825rem">
            <input type="checkbox" bind:checked={overrideExisting} />
            <span>Override existing details with metadata provider</span>
          </label>
          <span class="muted-xs" style="display:block;margin-left:1.5rem">
            When unchecked, metadata providers only fill in missing fields. When checked, provider metadata is replaced with the selected source.
          </span>

          <div style="display:flex;gap:0.5rem;margin-top:1rem;flex-wrap:wrap">
            <button
              type="button"
              class="btn btn-secondary"
              onclick={() => handleEnhanceMetadata(managingGame!)}
              disabled={isEnhancing}
            >
              <span class:spin={isEnhancing}>✨</span>
              <span>{isEnhancing ? "Scanning Metadata…" : "Enhance Metadata"}</span>
            </button>

            <button
              type="button"
              class="btn btn-ghost"
              onclick={() => handleResetMetadataCache(managingGame!)}
              title="Clears user/metadata-provider enhancements and restores default game provider data"
              disabled={loadingMeta}
            >
              ↺ Reset Cache to Provider Defaults
            </button>
          </div>
          <span class="muted-xs" style="margin-top:0.5rem;display:block">
            Note: Game providers are only allowed to override metadata from metadata providers when you choose to reset the metadata cache.
          </span>
        </div>
      </div>

      <div class="row modal-footer" style="justify-content:flex-end;gap:0.5rem;margin-top:1.5rem">
        <button class="btn btn-ghost" onclick={() => (showManageModal = false)}>Cancel</button>
        <button class="btn btn-primary" onclick={saveCustomName}>Save Changes</button>
      </div>
    </div>
  </div>
{/if}

<!-- Fullscreen Media Lightbox Modal -->
{#if activeLightboxMedia}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="lightbox-backdrop" onclick={() => (activeLightboxMedia = null)} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="lightbox-content" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
      <button class="lightbox-close-btn" onclick={() => (activeLightboxMedia = null)}>✕</button>
      <img src={activeLightboxMedia.url} alt={activeLightboxMedia.title || "Full resolution media"} />
      {#if activeLightboxMedia.title}
        <div class="lightbox-caption">{activeLightboxMedia.title}</div>
      {/if}
    </div>
  </div>
{/if}

{#if showAddGameModal}
  <AddGameModal onclose={() => (showAddGameModal = false)} />
{/if}

<style>
  .library-container {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 2rem;
  }

  .empty-library-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
    gap: 1rem;
    border-radius: var(--radius-md);
  }

  .empty-icon {
    font-size: 3rem;
  }

  .feedback-pill {
    animation: fadeIn 0.2s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .lightbox-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10000;
    background: rgba(0, 0, 0, 0.88);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    backdrop-filter: blur(8px);
  }

  .lightbox-content {
    position: relative;
    max-width: 90vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .lightbox-content img {
    max-width: 100%;
    max-height: 85vh;
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8);
  }

  .lightbox-close-btn {
    position: absolute;
    top: -36px;
    right: 0;
    background: transparent;
    border: none;
    color: #fff;
    font-size: 1.5rem;
    cursor: pointer;
  }

  .lightbox-caption {
    margin-top: 8px;
    color: var(--text-dim);
    font-size: 0.85rem;
  }
</style>
