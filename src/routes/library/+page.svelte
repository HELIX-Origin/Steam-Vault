<script lang="ts">
  import "$styles/library.scss";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t, locale, activeLanguage } from "$lib/i18n";
  import { customNames, error, games, loading, selectedGameId, settings } from "$lib/stores";
  import { getPlatformMetadata, pluginEngine } from "$lib/plugins";
  import PlatformIcon from "$lib/components/PlatformIcon.svelte";
  import AddGameModal from "$lib/components/library/AddGameModal.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
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
  let failedCoverAppId = $state<number | null>(null);

  // Per-game customization modal fields
  let modalCustomName = $state("");
  let modalShortDescription = $state("");
  let modalDevelopers = $state("");
  let modalPublishers = $state("");
  let modalReleaseDate = $state("");
  let modalGenres = $state("");
  let modalCapsuleImage = $state("");
  let modalHeroImage = $state("");
  let modalLogoImage = $state("");
  let modalFeedback = $state<string | null>(null);

  // Media Showcase & Audio Tabs State
  let activeMediaTab = $state<"screenshots" | "trailers" | "soundtracks">("screenshots");
  let selectedTrailerIdx = $state(0);
  let isTrailersAutoplay = $state(false);
  let isTrailersMuted = $state(true);
  let soundtrackAudioEl = $state<HTMLAudioElement | null>(null);
  let isPlayingSoundtrack = $state(false);
  let soundtrackVolume = $state(0.5);
  let isSoundtrackLooping = $state(true);
  let currentTrackIdx = $state(0);

  function togglePlaySoundtrack() {
    if (!soundtrackAudioEl) return;
    if (isPlayingSoundtrack) {
      soundtrackAudioEl.pause();
      isPlayingSoundtrack = false;
    } else {
      soundtrackAudioEl.volume = soundtrackVolume;
      soundtrackAudioEl.loop = isSoundtrackLooping;
      soundtrackAudioEl.play().then(() => {
        isPlayingSoundtrack = true;
      }).catch((e) => console.warn("Soundtrack playback error:", e));
    }
  }

  function handleVolumeChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    soundtrackVolume = val;
    if (soundtrackAudioEl) {
      soundtrackAudioEl.volume = val;
    }
  }

  function toggleLoopSoundtrack() {
    isSoundtrackLooping = !isSoundtrackLooping;
    if (soundtrackAudioEl) {
      soundtrackAudioEl.loop = isSoundtrackLooping;
    }
  }

  function getDisplayName(game: Game | null | undefined): string {
    if (!game) return "";
    return $customNames[game.appId] || game.name || `App ${game.appId}`;
  }

  function getLocalizedTitle(game: Game | null | undefined, meta?: AppMeta | null): string {
    if (!game) return "";
    if ($customNames[game.appId]) return $customNames[game.appId];
    if (meta?.userEnhanced && meta?.name) return meta.name;
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
    if (meta.userEnhanced && meta.shortDescription) return meta.shortDescription;
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

  function resolveArtworkSrc(raw?: string | null): string {
    if (!raw || !raw.trim()) return "";
    const trimmed = raw.trim();
    if (
      trimmed.startsWith("http://") ||
      trimmed.startsWith("https://") ||
      trimmed.startsWith("data:") ||
      trimmed.startsWith("asset:")
    ) {
      return trimmed;
    }
    try {
      return convertFileSrc(trimmed);
    } catch {
      return trimmed;
    }
  }

  function getGameCover(game: Game, meta?: AppMeta | null): string {
    if (meta?.capsuleImage) return resolveArtworkSrc(meta.capsuleImage);
    if (game.capsuleImage) return resolveArtworkSrc(game.capsuleImage);
    if (meta?.headerImage) return resolveArtworkSrc(meta.headerImage);
    if (game.platform === "steam" && game.appId < 50_000_000) return getCapsuleUrl(game.appId);
    if (game.icon && (game.icon.startsWith("http") || game.icon.startsWith("data:") || game.icon.startsWith("asset:"))) {
      return game.icon;
    }
    return "";
  }

  function getGameHero(game: Game, meta?: AppMeta | null): string {
    if (meta?.heroImage) return resolveArtworkSrc(meta.heroImage);
    if (game.heroImage) return resolveArtworkSrc(game.heroImage);
    if (game.platform === "steam" && game.appId < 50_000_000) return getHeroUrl(game.appId);
    if (meta?.capsuleImage) return resolveArtworkSrc(meta.capsuleImage);
    if (game.capsuleImage) return resolveArtworkSrc(game.capsuleImage);
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
    if (meta?.heroImage) artworkCandidates.push(resolveArtworkSrc(meta.heroImage));
    if (game.heroImage) artworkCandidates.push(resolveArtworkSrc(game.heroImage));
    if (game.platform === "steam" && game.appId < 50_000_000) artworkCandidates.push(getHeroUrl(game.appId));
    if (meta?.capsuleImage) artworkCandidates.push(resolveArtworkSrc(meta.capsuleImage));
    if (game.capsuleImage) artworkCandidates.push(resolveArtworkSrc(game.capsuleImage));
    if (game.platform === "steam" && game.appId < 50_000_000) artworkCandidates.push(getCapsuleUrl(game.appId));

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

  function handleGalleryImageError(failedUrl: string) {
    mediaItems = mediaItems.filter((item) => item.url !== failedUrl);
    if (selectedGalleryIdx >= mediaItems.length) {
      selectedGalleryIdx = Math.max(0, mediaItems.length - 1);
    }
  }

  import { backgroundWorker, withTimeout } from "$lib/worker";

  async function loadGameDetails(game: Game) {
    selectedGame = game;
    selectedGalleryIdx = 0;
    selectedTrailerIdx = 0;
    currentTrackIdx = 0;
    activeMediaTab = "screenshots";
    failedCoverAppId = null;
    if (soundtrackAudioEl) {
      soundtrackAudioEl.pause();
      isPlayingSoundtrack = false;
    }

    // 1. Load cached metadata immediately without blocking the UI
    try {
      let meta = await withTimeout(api.kvGetJson<AppMeta>(`game_meta:${game.appId}`), 2000, null);
      if (!meta) {
        meta = await withTimeout(
          api.getCachedGameMetadata(game.appId, game.name, game.exe),
          2000,
          null
        );
      }

      gameMeta = meta;
      mediaItems = buildMediaGallery(game, meta);
      loadingMeta = false;

      // 2. Automatic metadata & artwork enhancement via Universal Metadata Engine
      const hasPlaceholderDesc =
        meta?.shortDescription === "Local standalone executable game." ||
        (meta?.shortDescription?.startsWith("Emulated title for ") ?? false);
      const isMissingTrailersForSteam =
        (game.platform === "steam" || game.kind === "steam") &&
        (!meta?.movies || meta.movies.length === 0);
      const needsAutoEnhance =
        !meta ||
        hasPlaceholderDesc ||
        isMissingTrailersForSteam ||
        !meta.shortDescription ||
        !meta.heroImage ||
        !meta.capsuleImage ||
        !meta.developers?.length;

      if (needsAutoEnhance) {
        const targetId = game.appId;
        backgroundWorker.dispatch(
          `auto-meta:${targetId}`,
          `Automatically enhancing metadata for ${game.name}…`,
          async () => {
            const { meta: enhanced } = await backgroundWorker.enhanceGameMetadata(game, meta, {
              override: hasPlaceholderDesc,
            });
            if (enhanced && selectedGame?.appId === targetId) {
              gameMeta = enhanced;
              mediaItems = buildMediaGallery(game, enhanced);
            }
          }
        );
      }
    } catch {
      gameMeta = null;
      mediaItems = buildMediaGallery(game, null);
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
    modalCustomName = $customNames[game.appId] ?? gameMeta?.name ?? game.name;
    modalShortDescription = gameMeta?.shortDescription ?? "";
    modalDevelopers = (gameMeta?.developers ?? []).join(", ");
    modalPublishers = (gameMeta?.publishers ?? []).join(", ");
    modalReleaseDate = gameMeta?.releaseDate ?? (gameMeta?.releaseYear ? String(gameMeta.releaseYear) : "");
    modalGenres = (gameMeta?.genres ?? []).join(", ");
    modalCapsuleImage = gameMeta?.capsuleImage ?? game.capsuleImage ?? "";
    modalHeroImage = gameMeta?.heroImage ?? game.heroImage ?? "";
    modalLogoImage = gameMeta?.logoImage ?? game.logoImage ?? "";
    modalFeedback = null;
    showManageModal = true;
  }

  async function pickArtworkFile(slot: "capsule" | "hero" | "logo") {
    try {
      const picked = await api.pickGameImageFile();
      if (!picked) return;
      if (slot === "capsule") modalCapsuleImage = picked;
      else if (slot === "hero") modalHeroImage = picked;
      else if (slot === "logo") modalLogoImage = picked;
    } catch (e) {
      modalFeedback = `Could not pick image: ${errorMessage(e)}`;
    }
  }

  async function saveGameCustomizations() {
    if (!managingGame) return;
    const cleanName = modalCustomName.trim() || managingGame.name;

    customNames.update((names) => {
      const copy = { ...names };
      if (!cleanName || cleanName === managingGame?.name) {
        delete copy[managingGame!.appId];
      } else {
        copy[managingGame!.appId] = cleanName;
      }
      return copy;
    });

    const splitList = (raw: string) =>
      raw
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);

    const yearMatch = modalReleaseDate.match(/\b(19\d{2}|20\d{2})\b/);
    const parsedYear = yearMatch ? Number(yearMatch[1]) : (gameMeta?.releaseYear ?? null);

    const updatedMeta: AppMeta = {
      appId: managingGame.appId,
      name: cleanName,
      shortDescription: modalShortDescription.trim() || null,
      developers: splitList(modalDevelopers),
      publishers: splitList(modalPublishers),
      releaseDate: modalReleaseDate.trim() || null,
      releaseYear: parsedYear,
      genres: splitList(modalGenres),
      categories: gameMeta?.categories ?? [],
      tags: gameMeta?.tags ?? [],
      headerImage: gameMeta?.headerImage ?? null,
      capsuleImage: modalCapsuleImage.trim() || null,
      heroImage: modalHeroImage.trim() || null,
      logoImage: modalLogoImage.trim() || null,
      userScore: gameMeta?.userScore ?? null,
      fetchedAt: Date.now(),
      screenshots: gameMeta?.screenshots ?? [],
      movies: gameMeta?.movies ?? [],
      soundtracks: gameMeta?.soundtracks ?? [],
      userEnhanced: true,
      enhancedBy: Array.from(new Set([...(gameMeta?.enhancedBy ?? []), "User Customized"])),
    };

    try {
      await api.kvSetJson(`game_meta:${managingGame.appId}`, updatedMeta);
      if (selectedGame?.appId === managingGame.appId) {
        gameMeta = updatedMeta;
      }
      games.update((list) =>
        list.map((g) =>
          g.appId === managingGame!.appId
            ? {
                ...g,
                capsuleImage: updatedMeta.capsuleImage || g.capsuleImage,
                heroImage: updatedMeta.heroImage || g.heroImage,
                logoImage: updatedMeta.logoImage || g.logoImage,
              }
            : g
        )
      );
      modalFeedback = "Saved game customizations.";
      setTimeout(() => {
        showManageModal = false;
        managingGame = null;
        modalFeedback = null;
      }, 700);
    } catch (e) {
      modalFeedback = `Failed to save customizations: ${errorMessage(e)}`;
    }
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
      const universalPlatform = plugins.find(
        (p) => p.id === "platform-universal" || p.id === "platform-standalone"
      );
      isStandaloneEnabled = universalPlatform ? universalPlatform.enabled : true;

      const mediaCfg = await pluginEngine.loadPluginConfig("media-universal");
      isTrailersAutoplay = mediaCfg?.autoPlayTrailers === true;
      isTrailersMuted = mediaCfg?.muteTrailers !== false;
      soundtrackVolume = (mediaCfg?.ambientVolume ?? 50) / 100;
      isSoundtrackLooping = mediaCfg?.loopAmbient !== false;
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
    {@const coverUrl = getGameCover(selectedGame, gameMeta)}
    {@const heroUrl = getGameHero(selectedGame, gameMeta)}

    <!-- Top Action Bar -->
    <div class="details-topbar glass">
      <div class="topbar-title-block">
        <h2 class="topbar-game-name">{getLocalizedTitle(selectedGame, gameMeta)}</h2>
        <span class="pill pill-accent2" style="display:inline-flex;align-items:center;gap:0.35rem">
          <PlatformIcon platform={selectedGame.platform} consoleName={selectedGame.console} size={13} />
          {selectedGame.console || getPlatformMetadata(selectedGame.platform).label}
        </span>
        {#if selectedGame.installed}
          <span class="pill pill-ok">Installed</span>
        {:else}
          <span class="pill pill-dim">Not Installed</span>
        {/if}
        {#if loadingMeta}
          <span class="pill pill-dim" title="Automatically resolving metadata and artwork">
            <span class="spin" style="display:inline-block;margin-right:0.25rem">⟳</span> Syncing Metadata…
          </span>
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

        <button class="btn btn-ghost" onclick={() => openManageModal(selectedGame!)} title="Customize Game Details & Artwork">
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
        {#if coverUrl && failedCoverAppId !== selectedGame.appId}
          <img
            class="hero-capsule"
            src={coverUrl}
            alt={selectedGame.name}
            onerror={() => {
              if (selectedGame) failedCoverAppId = selectedGame.appId;
            }}
          />
        {:else}
          <div class="hero-capsule hero-capsule-fallback platform-{selectedGame.platform}">
            <span class="fallback-icon">
              <PlatformIcon platform={selectedGame.platform} consoleName={selectedGame.console} size={36} />
            </span>
            <span class="fallback-platform">{selectedGame.console || getPlatformMetadata(selectedGame.platform).label}</span>
          </div>
        {/if}

        <div class="hero-meta">
          <h1 class="hero-title">{getLocalizedTitle(selectedGame, gameMeta)}</h1>
          {#if getLocalizedDescription(gameMeta)}
            <p class="hero-desc">{getLocalizedDescription(gameMeta)}</p>
          {/if}
          <div class="row hero-tags">
            <span class="pill pill-accent" style="display:inline-flex;align-items:center;gap:0.35rem">
              <PlatformIcon platform={selectedGame.platform} consoleName={selectedGame.console} size={13} />
              {selectedGame.console || getPlatformMetadata(selectedGame.platform).label}
            </span>
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
      <!-- Unified Media Showcase (Tabs: Screenshots & Artwork, Video Trailers, Soundtracks) -->
      <section class="glass-strong card media-gallery-card">
        <div class="row-between section-head" style="align-items:center;flex-wrap:wrap;gap:0.5rem">
          <div class="row media-nav-tabs" style="gap:0.4rem;align-items:center">
            <button
              type="button"
              class="tab-btn"
              class:active={activeMediaTab === "screenshots"}
              onclick={() => (activeMediaTab = "screenshots")}
            >
              🖼️ Screenshots &amp; Art ({mediaItems.length})
            </button>

            {#if (gameMeta?.movies ?? []).length > 0}
              <button
                type="button"
                class="tab-btn"
                class:active={activeMediaTab === "trailers"}
                onclick={() => (activeMediaTab = "trailers")}
              >
                🎬 Trailers ({(gameMeta?.movies ?? []).length})
              </button>
            {/if}

            {#if (gameMeta?.soundtracks ?? []).length > 0}
              <button
                type="button"
                class="tab-btn"
                class:active={activeMediaTab === "soundtracks"}
                onclick={() => (activeMediaTab = "soundtracks")}
              >
                🎵 Soundtracks ({(gameMeta?.soundtracks ?? []).length})
                {#if isPlayingSoundtrack}
                  <span class="pulse-dot" style="display:inline-block;margin-left:4px"></span>
                {/if}
              </button>
            {/if}
          </div>

          {#if activeMediaTab === "screenshots" && mediaItems.length > 0}
            <span class="muted-sm">{selectedGalleryIdx + 1} / {mediaItems.length}</span>
          {:else if activeMediaTab === "trailers" && (gameMeta?.movies ?? []).length > 0}
            <span class="muted-sm">{selectedTrailerIdx + 1} / {(gameMeta?.movies ?? []).length}</span>
          {:else if activeMediaTab === "soundtracks" && isPlayingSoundtrack}
            <span class="pill pill-ok audio-indicator">
              <span class="pulse-dot"></span> Playing Audio
            </span>
          {/if}
        </div>

        {#if activeMediaTab === "screenshots"}
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
                <img
                  src={currentMedia.url}
                  alt={currentMedia.title || "Game media"}
                  onerror={() => handleGalleryImageError(currentMedia.url)}
                />
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
                  <img
                    src={item.url}
                    alt={item.title || `Thumbnail ${idx + 1}`}
                    onerror={() => handleGalleryImageError(item.url)}
                  />
                </button>
              {/each}
            </div>
          {:else}
            <div class="no-media-placeholder">
              <span class="muted">No media preview available for this title.</span>
            </div>
          {/if}
        {:else if activeMediaTab === "trailers"}
          {#if (gameMeta?.movies ?? []).length > 0}
            {@const activeTrailerUrl = (gameMeta?.movies ?? [])[selectedTrailerIdx]}
            <div class="trailer-player-container">
              <video
                src={activeTrailerUrl}
                controls
                playsinline
                autoplay={isTrailersAutoplay}
                muted={isTrailersMuted}
                class="trailer-video"
              >
                <track kind="captions" />
              </video>
            </div>

            {#if (gameMeta?.movies ?? []).length > 1}
              <div class="trailer-selector-list">
                {#each (gameMeta?.movies ?? []) as _movieUrl, idx}
                  <button
                    class="trailer-chip-btn"
                    class:active={selectedTrailerIdx === idx}
                    onclick={() => (selectedTrailerIdx = idx)}
                  >
                    <span class="chip-icon">▶</span> Trailer {idx + 1}
                  </button>
                {/each}
              </div>
            {/if}
          {:else}
            <div class="no-media-placeholder">
              <span class="muted">No video trailers found for this title.</span>
            </div>
          {/if}
        {:else if activeMediaTab === "soundtracks"}
          {#if (gameMeta?.soundtracks ?? []).length > 0}
            {@const activeTrackUrl = (gameMeta?.soundtracks ?? [])[currentTrackIdx]}
            <div class="soundtrack-player-box">
              <audio
                bind:this={soundtrackAudioEl}
                src={activeTrackUrl}
                onended={() => {
                  if (!isSoundtrackLooping) isPlayingSoundtrack = false;
                }}
              ></audio>

              <div class="audio-control-bar">
                <button class="btn btn-primary play-audio-btn" onclick={togglePlaySoundtrack}>
                  {isPlayingSoundtrack ? "⏸ Pause Theme" : "▶ Play Theme"}
                </button>

                <div class="audio-volume-control">
                  <span class="volume-icon">🔊</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={soundtrackVolume}
                    oninput={handleVolumeChange}
                    class="volume-slider"
                    title="Master Volume"
                  />
                  <span class="volume-pct">{Math.round(soundtrackVolume * 100)}%</span>
                </div>

                <button
                  class="btn mini btn-ghost loop-toggle-btn"
                  class:active={isSoundtrackLooping}
                  onclick={toggleLoopSoundtrack}
                  title="Seamless Loop"
                >
                  🔁 Loop
                </button>
              </div>

              {#if (gameMeta?.soundtracks ?? []).length > 1}
                <div class="soundtrack-tracklist" style="margin-top:0.75rem">
                  {#each (gameMeta?.soundtracks ?? []) as trackUrl, idx}
                    <button
                      class="track-item-btn"
                      class:active={currentTrackIdx === idx}
                      onclick={() => {
                        currentTrackIdx = idx;
                        if (isPlayingSoundtrack && soundtrackAudioEl) {
                          soundtrackAudioEl.src = trackUrl;
                          soundtrackAudioEl.play();
                        }
                      }}
                    >
                      <span>Track {idx + 1}</span>
                      <span class="muted-xs">{trackUrl.split("/").pop()}</span>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <div class="no-media-placeholder">
              <span class="muted">No soundtrack audio tracks loaded for this game.</span>
            </div>
          {/if}
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

<!-- Manage Game Modal (Per-Game Metadata & Artwork Customization) -->
{#if showManageModal && managingGame}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal-backdrop" onclick={() => (showManageModal = false)} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="modal-dialog glass-strong"
      style="max-width:620px;max-height:86vh;overflow-y:auto"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <div class="row-between modal-header">
        <div class="modal-title-group">
          <h2>Manage &amp; Customize Game</h2>
          <span class="muted-sm" style="display:inline-flex;align-items:center;gap:0.35rem">
            <PlatformIcon platform={managingGame.platform} consoleName={managingGame.console} size={13} />
            {managingGame.name} (App ID: {managingGame.appId})
          </span>
        </div>
        <button class="btn btn-ghost close-btn" onclick={() => (showManageModal = false)}>✕</button>
      </div>

      {#if modalFeedback}
        <div class="pill pill-accent" style="margin-bottom:1rem;display:block;text-align:center">
          {modalFeedback}
        </div>
      {/if}

      <div class="modal-body" style="display:flex;flex-direction:column;gap:0.85rem">
        <label class="form-group">
          <span class="form-label">Display Title:</span>
          <input
            type="text"
            class="input-text"
            bind:value={modalCustomName}
            placeholder={managingGame.name}
          />
          <span class="muted-xs">Customizes the title shown in the library sidebar, details view, and showcase.</span>
        </label>

        <label class="form-group">
          <span class="form-label">Short Description / Summary:</span>
          <textarea
            class="input-text"
            rows="3"
            style="resize:vertical;font-family:inherit"
            bind:value={modalShortDescription}
            placeholder="Enter a short summary or description for this game…"
          ></textarea>
        </label>

        <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem">
          <label class="form-group">
            <span class="form-label">Developers (comma-separated):</span>
            <input
              type="text"
              class="input-text"
              bind:value={modalDevelopers}
              placeholder="e.g. Nintendo EAD, Valve"
            />
          </label>

          <label class="form-group">
            <span class="form-label">Publishers (comma-separated):</span>
            <input
              type="text"
              class="input-text"
              bind:value={modalPublishers}
              placeholder="e.g. Nintendo, Devolver Digital"
            />
          </label>
        </div>

        <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem">
          <label class="form-group">
            <span class="form-label">Release Date / Year:</span>
            <input
              type="text"
              class="input-text"
              bind:value={modalReleaseDate}
              placeholder="e.g. 2002-12-13 or 2002"
            />
          </label>

          <label class="form-group">
            <span class="form-label">Genres (comma-separated):</span>
            <input
              type="text"
              class="input-text"
              bind:value={modalGenres}
              placeholder="e.g. Action, Adventure, RPG"
            />
          </label>
        </div>

        <div style="padding-top:0.75rem;border-top:1px solid rgba(255,255,255,0.08);display:flex;flex-direction:column;gap:0.75rem">
          <span class="form-label">Custom Artwork (URL or Local Image File)</span>

          <div class="form-group">
            <span class="muted-xs">Portrait Cover / Boxart (2:3 Capsule):</span>
            <div style="display:flex;gap:0.5rem;margin-top:0.25rem">
              <input
                type="text"
                class="input-text"
                style="flex:1"
                bind:value={modalCapsuleImage}
                placeholder="https://… or local file path"
              />
              <button type="button" class="btn btn-secondary" onclick={() => pickArtworkFile("capsule")}>
                📂 Browse…
              </button>
            </div>
          </div>

          <div class="form-group">
            <span class="muted-xs">Hero Backdrop Banner (16:9 Wide):</span>
            <div style="display:flex;gap:0.5rem;margin-top:0.25rem">
              <input
                type="text"
                class="input-text"
                style="flex:1"
                bind:value={modalHeroImage}
                placeholder="https://… or local file path"
              />
              <button type="button" class="btn btn-secondary" onclick={() => pickArtworkFile("hero")}>
                📂 Browse…
              </button>
            </div>
          </div>

          <div class="form-group">
            <span class="muted-xs">Transparent Title Logo (PNG / SVG):</span>
            <div style="display:flex;gap:0.5rem;margin-top:0.25rem">
              <input
                type="text"
                class="input-text"
                style="flex:1"
                bind:value={modalLogoImage}
                placeholder="https://… or local file path"
              />
              <button type="button" class="btn btn-secondary" onclick={() => pickArtworkFile("logo")}>
                📂 Browse…
              </button>
            </div>
          </div>
        </div>

        <div style="padding-top:0.75rem;border-top:1px solid rgba(255,255,255,0.08)">
          <span class="form-label">Platform &amp; Launch Target</span>
          <div class="code-val" style="word-break:break-all;margin-top:0.35rem">
            {managingGame.launchUri || managingGame.exe || "Platform default"}
          </div>
          {#if gameMeta?.enhancedBy && gameMeta.enhancedBy.length > 0}
            <span class="muted-xs" style="display:block;margin-top:0.4rem">
              Metadata sources: {gameMeta.enhancedBy.join(", ")}. Global metadata cache refresh and purge controls are located in Settings → Storage &amp; Cache.
            </span>
          {/if}
        </div>
      </div>

      <div class="row modal-footer" style="justify-content:flex-end;gap:0.5rem;margin-top:1.25rem">
        <button class="btn btn-ghost" onclick={() => (showManageModal = false)}>Cancel</button>
        <button class="btn btn-primary" onclick={saveGameCustomizations}>Save Customizations</button>
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
