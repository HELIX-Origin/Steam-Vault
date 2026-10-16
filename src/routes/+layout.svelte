<script lang="ts">
  import "$styles/app.scss";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { isRTL, syncWithSettingsLanguage, t } from "$lib/i18n";
  import {
    customNames,
    error,
    games,
    platformScan,
    providers,
    selectedGameId,
    settings,
    steamRunning
  } from "$lib/stores";
  import type { Game } from "$lib/types";

  let { children } = $props();

  let splashVisible = $state(true);
  let splashExiting = $state(false);
  let isScanning = $state(false);

  // Steam/GOG Sidebar Game List Filter State
  let sidebarQuery = $state("");
  let sidebarFilterMode = $state<"all" | "installed" | "favorites">("all");

  const titlebarNav = [
    { href: "/", label: "Dashboard" },
    { href: "/library", label: "Library" },
    { href: "/platforms", label: "Platforms" },
    { href: "/storage", label: "Storage" },
    { href: "/settings", label: "Settings" }
  ];

  const platformTagMap: Record<string, string> = {
    steam: "STM",
    gog: "GOG",
    epic: "EAS",
    itch: "ITC",
    ubisoft: "UBI",
    ea: "EA",
    battlenet: "BNET",
    amazon: "AMZ",
    xbox: "XBO",
    rockstar: "RST",
    other: "OTH"
  };

  function getPlatformTag(platform: string): string {
    return platformTagMap[platform.toLowerCase()] || platform.slice(0, 3).toUpperCase();
  }

  function getDisplayName(game: Game): string {
    return $customNames[game.appId] ?? game.name;
  }

  let installedCount = $derived($games.filter((g) => g.installed).length);
  let favoriteCount = $derived($games.filter((g) => g.favorite).length);

  let filteredGames = $derived(
    $games.filter((game) => {
      if (sidebarFilterMode === "installed" && !game.installed) return false;
      if (sidebarFilterMode === "favorites" && !game.favorite) return false;
      if (sidebarQuery.trim()) {
        const query = sidebarQuery.trim().toLowerCase();
        const name = getDisplayName(game).toLowerCase();
        const plat = (game.platform || "").toLowerCase();
        return name.includes(query) || plat.includes(query);
      }
      return true;
    })
  );

  function selectGame(game: Game) {
    selectedGameId.set(game.appId);
    if (page.url.pathname !== "/library") {
      goto("/library");
    }
  }

  async function handleScanAllPlatforms() {
    if (isScanning) return;
    isScanning = true;
    try {
      const scan = await api.scanAllPlatforms();
      platformScan.set(scan);
      const updated = await api.loadLibrary(undefined, true);
      games.set(updated);
      if (!$selectedGameId && updated.length > 0) {
        selectedGameId.set(updated[0].appId);
      }
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      isScanning = false;
    }
  }

  onMount(async () => {
    const minSplashDuration = new Promise((resolve) => setTimeout(resolve, 850));
    try {
      providers.set(await api.getProviders());
      const loadedSettings = await api.getSettings();
      settings.set(loadedSettings);
      syncWithSettingsLanguage(loadedSettings?.language);
      steamRunning.set(await api.isSteamRunning());

      // Initial library load if empty
      const initialGames = await api.loadLibrary();
      games.set(initialGames);
      if (!$selectedGameId && initialGames.length > 0) {
        selectedGameId.set(initialGames[0].appId);
      }
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      await minSplashDuration;
      splashExiting = true;
      setTimeout(() => {
        splashVisible = false;
      }, 500);
    }
  });
</script>

<div class="app" dir={$isRTL ? "rtl" : "ltr"}>
  <!-- Steam-style Titlebar with Brand, Top Navigation Links, and Window Controls -->
  <header class="titlebar" data-tauri-drag-region>
    <div class="tb-brand" data-tauri-drag-region>
      <div class="tb-mark" data-tauri-drag-region>
        <img src="/logo.svg" alt="Universal Game Launcher" width="28" height="28" />
      </div>
      <div class="tb-titles" data-tauri-drag-region>
        <span class="tb-title" data-tauri-drag-region>{$t("nav.brandTitle")}</span>
        <span class="tb-sub" data-tauri-drag-region>{$t("nav.brandSubtitle")}</span>
      </div>
    </div>

    <!-- Title Bar Navigation Links (Steam / GOG Top Menu Style) -->
    <nav class="tb-nav" data-tauri-drag-region="false">
      {#each titlebarNav as item (item.href)}
        <a
          class="tb-nav-link"
          class:active={page.url.pathname === item.href || (item.href !== "/" && page.url.pathname.startsWith(item.href))}
          href={item.href}
          data-tauri-drag-region="false"
        >
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="tb-spacer" data-tauri-drag-region></div>
    <WindowControls position="right" />
  </header>

  <div class="shell">
    <!-- Steam / GOG Galaxy Left Game List Sidebar -->
    <aside class="sidebar glass">
      <div class="sidebar-top">
        <div class="sidebar-search-row">
          <div class="sidebar-search-wrap">
            <span class="search-icon">🔍</span>
            <input
              type="text"
              class="sidebar-search-input"
              placeholder="Filter games..."
              bind:value={sidebarQuery}
            />
            {#if sidebarQuery}
              <button
                class="search-clear-btn"
                onclick={() => (sidebarQuery = "")}
                title="Clear filter"
              >
                ×
              </button>
            {/if}
          </div>
          <button
            class="scan-action-btn"
            class:spinning={isScanning}
            onclick={handleScanAllPlatforms}
            title="Scan all platforms for installed games and updates"
            disabled={isScanning}
          >
            ↻
          </button>
        </div>

        <!-- Quick Filter Pills -->
        <div class="sidebar-filters">
          <button
            class="filter-pill"
            class:active={sidebarFilterMode === "all"}
            onclick={() => (sidebarFilterMode = "all")}
          >
            All <span class="pill-count">{$games.length}</span>
          </button>
          <button
            class="filter-pill"
            class:active={sidebarFilterMode === "installed"}
            onclick={() => (sidebarFilterMode = "installed")}
          >
            Installed <span class="pill-count">{installedCount}</span>
          </button>
          <button
            class="filter-pill"
            class:active={sidebarFilterMode === "favorites"}
            onclick={() => (sidebarFilterMode = "favorites")}
          >
            ★ <span class="pill-count">{favoriteCount}</span>
          </button>
        </div>
      </div>

      <!-- Live Game List -->
      <div class="sidebar-game-list">
        {#if filteredGames.length === 0}
          <div class="sidebar-empty">
            {#if sidebarQuery}
              <span>No matching games</span>
            {:else if $games.length === 0}
              <span>No games found.<br />Click ↻ Scan above.</span>
            {:else}
              <span>No games in this filter</span>
            {/if}
          </div>
        {:else}
          {#each filteredGames as game (game.kind + "-" + game.appId)}
            <button
              class="game-list-item"
              class:selected={game.appId === $selectedGameId}
              class:installed={game.installed}
              onclick={() => selectGame(game)}
            >
              <span class="status-dot" class:is-installed={game.installed}></span>
              <span class="platform-badge platform-{game.platform}">
                {getPlatformTag(game.platform)}
              </span>
              <span class="game-item-title" title={getDisplayName(game)}>
                {getDisplayName(game)}
              </span>
              {#if game.favorite}
                <span class="game-item-fav">★</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <div class="sidebar-footer">
        <span class="sidebar-status-note">
          {installedCount} of {$games.length} installed
        </span>
      </div>
    </aside>

    <main class="content">
      {#if $error}
        <div class="error-banner" style="margin-bottom:1rem">
          {$error}
          <button class="btn btn-ghost" style="float:right" onclick={() => error.set(null)}>
            {$t("status.dismiss")}
          </button>
        </div>
      {/if}
      {@render children?.()}
    </main>
  </div>
</div>

{#if splashVisible}
  <div class="splash-screen" class:exiting={splashExiting} role="dialog" aria-label="Launching Universal Game Launcher">
    <div class="splash-backdrop-glow"></div>
    <div class="splash-content">
      <div class="splash-logo-box">
        <div class="splash-pulse-ring"></div>
        <img src="/logo.svg" alt="Universal Game Launcher" class="splash-logo-img" width="84" height="84" />
      </div>
      <div class="splash-text-block">
        <h1 class="splash-title">Universal Game Launcher</h1>
        <p class="splash-sub">All your games. One high-performance launcher.</p>
      </div>
      <div class="splash-progress">
        <div class="splash-progress-bar"></div>
      </div>
    </div>
  </div>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .titlebar {
    flex: 0 0 auto;
    height: 48px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 12px 0 14px;
    user-select: none;
    border-bottom: 1px solid var(--border);
    background: #0c071d;
    box-shadow: 0 1px 0 0 rgba(255, 255, 255, 0.04);
    z-index: 10;
  }

  .tb-brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-left: 4px;
  }

  .tb-mark {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    overflow: hidden;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  }

  .tb-mark img {
    width: 100%;
    height: 100%;
    display: block;
  }

  .tb-titles {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .tb-title {
    font-size: 0.92rem;
    font-weight: 700;
    letter-spacing: -0.015em;
    color: var(--text);
  }

  .tb-sub {
    font-size: 0.74rem;
    color: var(--text-mute);
  }

  /* Steam-style Title Bar Navigation Links */
  .tb-nav {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 12px;
  }

  .tb-nav-link {
    display: inline-flex;
    align-items: center;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    color: #c4b5fd;
    font-size: 0.82rem;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-decoration: none;
    transition: all 0.16s ease;
    border: 1px solid transparent;
  }

  .tb-nav-link:hover {
    color: #ffffff;
    background: rgba(168, 85, 247, 0.14);
    border-color: rgba(168, 85, 247, 0.28);
  }

  .tb-nav-link.active {
    color: #ffffff;
    background: linear-gradient(135deg, rgba(168, 85, 247, 0.32), rgba(232, 121, 249, 0.22));
    border-color: rgba(192, 132, 252, 0.55);
    box-shadow: 0 2px 12px rgba(168, 85, 247, 0.25);
  }

  .tb-spacer {
    flex: 1;
  }

  .shell {
    display: grid;
    grid-template-columns: 280px 1fr;
    flex: 1;
    min-height: 0;
    padding: 12px;
    gap: 12px;
  }

  /* Steam/GOG Left Game List Sidebar */
  .sidebar {
    display: flex;
    flex-direction: column;
    padding: 12px 10px;
    gap: 12px;
    box-shadow: var(--glass-highlight), var(--shadow);
    overflow: hidden;
  }

  .sidebar-top {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;
  }

  .sidebar-search-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sidebar-search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    background: rgba(15, 8, 30, 0.7);
    border: 1px solid rgba(168, 85, 247, 0.25);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    gap: 6px;
    transition: border-color 0.15s ease;
  }

  .sidebar-search-wrap:focus-within {
    border-color: rgba(192, 132, 252, 0.7);
    box-shadow: 0 0 8px rgba(168, 85, 247, 0.3);
  }

  .search-icon {
    font-size: 0.75rem;
    opacity: 0.6;
  }

  .sidebar-search-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 0.8rem;
    padding: 2px 0;
  }

  .sidebar-search-input::placeholder {
    color: var(--text-mute);
  }

  .search-clear-btn {
    background: transparent;
    border: none;
    color: var(--text-mute);
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 2px;
  }

  .search-clear-btn:hover {
    color: #ffffff;
  }

  .scan-action-btn {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: rgba(168, 85, 247, 0.15);
    border: 1px solid rgba(168, 85, 247, 0.35);
    color: #e9d5ff;
    font-size: 1.15rem;
    font-weight: 700;
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: all 0.16s ease;
    flex-shrink: 0;
  }

  .scan-action-btn:hover:not(:disabled) {
    background: rgba(168, 85, 247, 0.32);
    border-color: rgba(192, 132, 252, 0.65);
    color: #ffffff;
    box-shadow: 0 0 10px rgba(168, 85, 247, 0.35);
  }

  .scan-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .scan-action-btn.spinning {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .sidebar-filters {
    display: flex;
    gap: 4px;
  }

  .filter-pill {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--text-dim);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.14s ease;
  }

  .filter-pill:hover {
    background: rgba(168, 85, 247, 0.12);
    color: #ffffff;
    border-color: rgba(168, 85, 247, 0.25);
  }

  .filter-pill.active {
    background: rgba(168, 85, 247, 0.28);
    color: #ffffff;
    border-color: rgba(192, 132, 252, 0.55);
  }

  .pill-count {
    font-size: 0.68rem;
    opacity: 0.7;
    background: rgba(0, 0, 0, 0.25);
    padding: 1px 4px;
    border-radius: 4px;
  }

  .sidebar-game-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-right: 2px;
  }

  .sidebar-empty {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-mute);
    font-size: 0.8rem;
    line-height: 1.5;
  }

  .game-list-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
    width: 100%;
    user-select: none;
  }

  .game-list-item:hover {
    background: rgba(168, 85, 247, 0.12);
    color: #ffffff;
    border-color: rgba(168, 85, 247, 0.22);
  }

  .game-list-item.selected {
    background: linear-gradient(90deg, rgba(168, 85, 247, 0.32), rgba(232, 121, 249, 0.18));
    color: #ffffff;
    border-color: rgba(192, 132, 252, 0.5);
    box-shadow: 0 1px 8px rgba(168, 85, 247, 0.2);
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.18);
    flex-shrink: 0;
  }

  .status-dot.is-installed {
    background: #4ade80;
    box-shadow: 0 0 6px rgba(74, 222, 128, 0.6);
  }

  .platform-badge {
    font-size: 0.62rem;
    font-weight: 750;
    letter-spacing: 0.04em;
    padding: 1px 4px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.07);
    color: #cbd5e1;
    flex-shrink: 0;
  }

  .platform-badge.platform-steam {
    background: rgba(56, 189, 248, 0.2);
    color: #7dd3fc;
  }

  .platform-badge.platform-gog {
    background: rgba(168, 85, 247, 0.2);
    color: #d8b4fe;
  }

  .platform-badge.platform-epic {
    background: rgba(244, 63, 94, 0.2);
    color: #fda4af;
  }

  .platform-badge.platform-itch {
    background: rgba(239, 68, 68, 0.2);
    color: #fca5a5;
  }

  .platform-badge.platform-xbox {
    background: rgba(34, 197, 94, 0.2);
    color: #86efac;
  }

  .game-item-title {
    flex: 1;
    font-size: 0.8rem;
    font-weight: 550;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .game-item-fav {
    color: #facc15;
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .sidebar-footer {
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .sidebar-status-note {
    font-size: 0.72rem;
    color: var(--text-mute);
  }

  .content {
    overflow-y: auto;
    padding: 4px 8px 40px;
    min-height: 0;
  }

  /* Animated Splash Screen Transition */
  .splash-screen {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(ellipse at center, #1b0736 0%, #0e041d 55%, #07010e 100%);
    transition: opacity 0.45s cubic-bezier(0.16, 1, 0.3, 1), transform 0.45s cubic-bezier(0.16, 1, 0.3, 1);
    user-select: none;
  }

  .splash-screen.exiting {
    opacity: 0;
    transform: scale(1.03);
    pointer-events: none;
  }

  .splash-backdrop-glow {
    position: absolute;
    width: 480px;
    height: 480px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(168, 85, 247, 0.28) 0%, rgba(232, 121, 249, 0.12) 35%, transparent 70%);
    pointer-events: none;
    animation: splashGlowPulse 2.8s ease-in-out infinite;
  }

  .splash-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.25rem;
    animation: splashContentEntrance 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .splash-logo-box {
    position: relative;
    width: 96px;
    height: 96px;
    display: grid;
    place-items: center;
  }

  .splash-pulse-ring {
    position: absolute;
    inset: -6px;
    border-radius: 28px;
    border: 2px solid rgba(192, 132, 252, 0.6);
    box-shadow: 0 0 24px rgba(168, 85, 247, 0.5), inset 0 0 16px rgba(232, 121, 249, 0.25);
    animation: splashRingPulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  .splash-logo-img {
    position: relative;
    z-index: 2;
    filter: drop-shadow(0 4px 16px rgba(0, 0, 0, 0.6));
  }

  .splash-text-block {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .splash-title {
    font-size: 1.55rem;
    font-weight: 750;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, #ffffff 30%, #e9d5ff 70%, #c084fc 100%);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    margin: 0;
  }

  .splash-sub {
    font-size: 0.88rem;
    color: rgba(233, 213, 255, 0.75);
    margin: 0;
  }

  .splash-progress {
    width: 180px;
    height: 3px;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
    position: relative;
    margin-top: 0.5rem;
  }

  .splash-progress-bar {
    position: absolute;
    inset-block: 0;
    width: 50%;
    background: linear-gradient(90deg, #38bdf8, #a855f7, #e879f9);
    border-radius: 9999px;
    box-shadow: 0 0 10px rgba(168, 85, 247, 0.8);
    animation: splashSweep 1.2s ease-in-out infinite;
  }

  @keyframes splashContentEntrance {
    from {
      opacity: 0;
      transform: translateY(14px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes splashRingPulse {
    0%, 100% {
      transform: scale(1);
      opacity: 0.8;
    }
    50% {
      transform: scale(1.06);
      opacity: 0.4;
    }
  }

  @keyframes splashGlowPulse {
    0%, 100% {
      transform: scale(1);
      opacity: 0.7;
    }
    50% {
      transform: scale(1.15);
      opacity: 0.95;
    }
  }

  @keyframes splashSweep {
    0% {
      left: -50%;
    }
    100% {
      left: 100%;
    }
  }
</style>
