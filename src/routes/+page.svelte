<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { customNames, error, games, install, loading, settings } from "$lib/stores";
  import type { DiskInfo, Game, Platform, StorageReport } from "$lib/types";
  import "$styles/dashboard.scss";

  let storageReport = $state<StorageReport | null>(null);
  let launchingAppId = $state<number | null>(null);
  let launchMsg = $state<string | null>(null);

  const PLATFORM_META: Record<Platform, { label: string; icon: string; color: string }> = {
    steam: { label: "Steam", icon: "♨", color: "#38bdf8" },
    gog: { label: "GOG Galaxy", icon: "◈", color: "#a855f7" },
    epic: { label: "Epic Games", icon: "▲", color: "#facc15" },
    xbox: { label: "Xbox / MS Store", icon: "⨂", color: "#22c55e" },
    ea: { label: "EA App", icon: "●", color: "#f97316" },
    ubisoft: { label: "Ubisoft Connect", icon: "🌀", color: "#06b6d4" },
    battlenet: { label: "Battle.net", icon: "❄", color: "#3b82f6" },
    itch: { label: "itch.io", icon: "⌗", color: "#ef4444" },
    amazon: { label: "Amazon Games", icon: "📦", color: "#eab308" },
    rockstar: { label: "Rockstar Games", icon: "★", color: "#fbbf24" },
    emulator: { label: "Emulators", icon: "🕹", color: "#ec4899" },
    other: { label: "Other", icon: "⚙", color: "#94a3b8" }
  };

  onMount(async () => {
    await refresh();
  });

  async function refresh() {
    loading.set(true);
    try {
      const [steamInstall, lib, report] = await Promise.all([
        api.detectSteam().catch(() => null),
        api.loadLibrary().catch(() => []),
        api.getStorageReport().catch(() => null)
      ]);
      if (steamInstall) install.set(steamInstall);
      games.set(lib);
      storageReport = report;
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      loading.set(false);
    }
  }

  // Library computations
  let installedGames = $derived($games.filter((g) => g.installed));
  let favorites = $derived($games.filter((g) => g.favorite));
  let totalPlaytimeMinutes = $derived(
    $games.reduce((acc, g) => acc + (g.playtimeMinutes || 0), 0)
  );

  let totalStorageBytes = $derived(
    installedGames.reduce((acc, g) => acc + (g.sizeOnDisk || 0), 0)
  );

  let disks = $derived<DiskInfo[]>(storageReport?.disks ?? []);
  let totalFreeDiskSpace = $derived(disks.reduce((sum, d) => sum + d.free, 0));

  // Jump Back In / Recently Played & Favorites (Installed prioritized)
  let jumpBackInGames = $derived.by<Game[]>(() => {
    // 1. Installed with recent playtime
    const recentInstalled = installedGames
      .filter((g) => g.lastPlayed && g.lastPlayed > 0)
      .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0));

    if (recentInstalled.length >= 4) return recentInstalled.slice(0, 6);

    // 2. Installed favorites
    const favInstalled = installedGames.filter((g) => g.favorite && !recentInstalled.includes(g));
    const combined = [...recentInstalled, ...favInstalled];
    if (combined.length >= 4) return combined.slice(0, 6);

    // 3. Any installed games by playtime
    const anyInstalled = installedGames
      .filter((g) => !combined.includes(g))
      .sort((a, b) => (b.playtimeMinutes ?? 0) - (a.playtimeMinutes ?? 0));

    return [...combined, ...anyInstalled].slice(0, 6);
  });

  // Top 5 played games
  let topPlayedGames = $derived.by<Game[]>(() => {
    return [...$games]
      .filter((g) => (g.playtimeMinutes || 0) > 0)
      .sort((a, b) => (b.playtimeMinutes || 0) - (a.playtimeMinutes || 0))
      .slice(0, 5);
  });

  let maxTopPlaytime = $derived(
    topPlayedGames.length > 0 ? topPlayedGames[0].playtimeMinutes || 1 : 1
  );

  // Platform breakdown distribution
  interface PlatformDistItem {
    platform: Platform;
    label: string;
    icon: string;
    color: string;
    count: number;
    installedCount: number;
    pct: number;
  }

  let platformDistribution = $derived.by<PlatformDistItem[]>(() => {
    const total = $games.length || 1;
    const platforms = Object.keys(PLATFORM_META) as Platform[];
    return platforms
      .map((p) => {
        const platformGames = $games.filter((g) => g.platform === p);
        const count = platformGames.length;
        const instCount = platformGames.filter((g) => g.installed).length;
        return {
          platform: p,
          label: PLATFORM_META[p].label,
          icon: PLATFORM_META[p].icon,
          color: PLATFORM_META[p].color,
          count,
          installedCount: instCount,
          pct: (count / total) * 100
        };
      })
      .filter((p) => p.count > 0)
      .sort((a, b) => b.count - a.count);
  });

  function getDisplayName(game: Game): string {
    return $customNames[game.appId] || game.name || `App ${game.appId}`;
  }

  function getCoverUrl(game: Game): string | null {
    if (game.kind === "steam" || game.appId < 0x80000000) {
      return `https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/${game.appId}/library_600x900_2x.jpg`;
    }
    return null;
  }

  function formatPlaytime(minutes: number | null | undefined): string {
    if (!minutes) return "0h";
    const hours = Math.floor(minutes / 60);
    return hours >= 1 ? `${hours.toLocaleString()}h` : `${minutes}m`;
  }

  function formatLastPlayed(timestamp: number | null | undefined): string {
    if (!timestamp) return "Never played";
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.max(0, now - timestamp);
    if (diff < 3600) return "Just now";
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric"
    });
  }

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

  async function handleQuickLaunch(game: Game, e?: Event) {
    if (e) {
      e.preventDefault();
      e.stopPropagation();
    }
    launchingAppId = game.appId;
    launchMsg = `Launching ${getDisplayName(game)}…`;
    try {
      await api.launchGame(game.appId, game.launchUri, game.exe);
      launchMsg = `Launched ${getDisplayName(game)}!`;
      setTimeout(() => {
        if (launchMsg?.includes("Launched")) launchMsg = null;
      }, 3500);
    } catch (err) {
      launchMsg = `Launch failed: ${errorMessage(err)}`;
    } finally {
      launchingAppId = null;
    }
  }

  let activeAccountName = $derived.by<string>(() => {
    if ($install?.accounts && $install.accounts.length > 0) {
      const active = $install.accounts[0];
      return active.personaName || active.accountName || "Gamer";
    }
    return "Gamer";
  });
</script>

<div class="dashboard-container">
  <!-- Top Header -->
  <div class="dashboard-header">
    <div>
      <h1>Unified Library Dashboard</h1>
      <p>Welcome back, <strong>{activeAccountName}</strong>. Here is your unified gaming overview across all storefronts.</p>
    </div>
    <div class="header-actions">
      {#if launchMsg}
        <span class="pill pill-glow">{launchMsg}</span>
      {/if}
      <button class="btn btn-ghost" onclick={refresh} disabled={$loading}>
        <span class:spin={$loading}>🔄</span>
        <span>{$loading ? $t("common.refreshing") : $t("common.refresh")}</span>
      </button>
      <a href="/library" class="btn btn-primary">
        Browse Library →
      </a>
    </div>
  </div>

  <!-- KPI Metrics Grid -->
  <div class="dashboard-kpi-grid">
    <div class="glass card dashboard-kpi-card">
      <span class="kpi-title">Total Library</span>
      <span class="kpi-value accented">{$games.length}</span>
      <span class="kpi-sub">Across {platformDistribution.length} platforms</span>
    </div>
    <div class="glass card dashboard-kpi-card">
      <span class="kpi-title">Installed &amp; Ready</span>
      <span class="kpi-value">{installedGames.length}</span>
      <span class="kpi-sub">{formatBytes(totalStorageBytes)} disk used</span>
    </div>
    <div class="glass card dashboard-kpi-card">
      <span class="kpi-title">Total Playtime</span>
      <span class="kpi-value">{formatPlaytime(totalPlaytimeMinutes)}</span>
      <span class="kpi-sub">Logged across library</span>
    </div>
    <div class="glass card dashboard-kpi-card">
      <span class="kpi-title">Favorites</span>
      <span class="kpi-value">{favorites.length}</span>
      <span class="kpi-sub">Starred titles</span>
    </div>
    <div class="glass card dashboard-kpi-card">
      <span class="kpi-title">Free Storage</span>
      <span class="kpi-value">{formatBytes(totalFreeDiskSpace)}</span>
      <span class="kpi-sub">Across {disks.length} drives</span>
    </div>
  </div>

  <!-- Jump Back In / Recently Played Carousel / Grid -->
  {#if jumpBackInGames.length > 0}
    <div>
      <div class="dash-section-head">
        <div>
          <h2>Jump Back In</h2>
          <p class="muted-sm">Recently played and favorite titles installed on this machine.</p>
        </div>
        <a href="/library?filter=installed" class="section-link">View all installed ({installedGames.length}) →</a>
      </div>

      <div class="recent-games-grid">
        {#each jumpBackInGames as game (game.platform + "-" + game.appId)}
          {@const pMeta = PLATFORM_META[game.platform] || PLATFORM_META.other}
          {@const coverSrc = getCoverUrl(game)}
          <a href={`/library?game=${game.appId}`} class="recent-game-card">
            <div class="recent-cover-wrap">
              {#if coverSrc}
                <img src={coverSrc} alt={getDisplayName(game)} loading="lazy" />
              {:else}
                <div class="fallback-cover" style="color:{pMeta.color}">
                  {pMeta.icon}
                </div>
              {/if}
            </div>

            <div class="recent-meta">
              <span class="recent-title" title={getDisplayName(game)}>{getDisplayName(game)}</span>
              <span class="recent-time">
                {game.lastPlayed ? formatLastPlayed(game.lastPlayed) : formatPlaytime(game.playtimeMinutes)}
              </span>
              <div class="recent-badge-row">
                <span class="pill pill-dim" style="border-color:{pMeta.color}40;color:{pMeta.color};font-size:0.68rem;padding:0.1rem 0.35rem">
                  {pMeta.icon} {pMeta.label}
                </span>
                {#if game.installed}
                  <span class="pill pill-ok" style="font-size:0.65rem;padding:0.1rem 0.3rem">Ready</span>
                {/if}
              </div>
            </div>

            {#if game.installed}
              <button
                class="recent-play-btn"
                title="Play {getDisplayName(game)}"
                onclick={(e) => handleQuickLaunch(game, e)}
                disabled={launchingAppId === game.appId}
              >
                ▶
              </button>
            {/if}
          </a>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Multi-Column Insights: Storefront Distribution & Storage Snapshot -->
  <div class="dash-columns">
    <!-- Left Column: Storefront Platform Ecosystem -->
    <div class="dash-col">
      <div class="platform-dist-card">
        <div class="dash-section-head" style="margin-bottom:0">
          <div>
            <h2>Storefront Ecosystem</h2>
            <p class="muted-sm">Multi-platform game ownership and library breakdown.</p>
          </div>
          <span class="muted-sm">{$games.length} titles</span>
        </div>

        <!-- Stacked Distribution Bar -->
        <div class="platform-stacked-bar">
          {#each platformDistribution as p}
            <div
              class="bar-segment"
              style="width:{p.pct}%;background:{p.color}"
              title="{p.label}: {p.count} games ({p.pct.toFixed(1)}%)"
            ></div>
          {/each}
        </div>

        <!-- Platform Interactive Legend -->
        <div class="platform-legend-grid">
          {#each platformDistribution as p}
            <a href={`/library?platform=${p.platform}`} class="platform-legend-item">
              <span class="legend-dot" style="background:{p.color}"></span>
              <span class="legend-name">{p.label}</span>
              <span class="legend-count">{p.count}</span>
            </a>
          {/each}
        </div>
      </div>

      <!-- Quick Actions Hub -->
      <div>
        <div class="dash-section-head">
          <h2>Quick Actions</h2>
        </div>
        <div class="quick-actions-grid">
          <a href="/library" class="quick-action-tile">
            <div class="tile-head">
              <span class="tile-icon">🎮</span>
              <span class="tile-arrow">→</span>
            </div>
            <span class="tile-title">Browse Library</span>
            <span class="tile-desc">Filter, sort, and launch your games across all collections.</span>
          </a>

          <a href="/storage" class="quick-action-tile">
            <div class="tile-head">
              <span class="tile-icon">💾</span>
              <span class="tile-arrow">→</span>
            </div>
            <span class="tile-title">Storage Analytics</span>
            <span class="tile-desc">Inspect disk usage, drive capacities, and per-platform footprints.</span>
          </a>

          <a href="/settings" class="quick-action-tile">
            <div class="tile-head">
              <span class="tile-icon">⬡</span>
              <span class="tile-arrow">→</span>
            </div>
            <span class="tile-title">External Launchers</span>
            <span class="tile-desc">Configure Legendary CLI, discover GOG, Epic, and ROM directories.</span>
          </a>

          <a href="/settings" class="quick-action-tile">
            <div class="tile-head">
              <span class="tile-icon">⚙️</span>
              <span class="tile-arrow">→</span>
            </div>
            <span class="tile-title">App Settings</span>
            <span class="tile-desc">Manage Steam sync, languages, API keys, and cache reset.</span>
          </a>
        </div>
      </div>
    </div>

    <!-- Right Column: Top Played Leaderboard & Storage Snapshot -->
    <div class="dash-col">
      <!-- Top Played Titles Leaderboard -->
      {#if topPlayedGames.length > 0}
        <div class="leaderboard-card">
          <div class="dash-section-head" style="margin-bottom:0">
            <div>
              <h2>Top Played Games</h2>
              <p class="muted-sm">Your most played games ranked by logged playtime.</p>
            </div>
            <span class="muted-sm">{formatPlaytime(totalPlaytimeMinutes)} total</span>
          </div>

          <div class="leaderboard-list">
            {#each topPlayedGames as game, idx (game.platform + "-" + game.appId)}
              {@const pMeta = PLATFORM_META[game.platform] || PLATFORM_META.other}
              <a href={`/library?game=${game.appId}`} class="leaderboard-item">
                <span class="leader-rank">#{idx + 1}</span>
                <span class="leader-name" title={getDisplayName(game)}>
                  {getDisplayName(game)}
                  <span class="pill pill-dim" style="font-size:0.65rem;margin-left:0.3rem;border-color:{pMeta.color}40;color:{pMeta.color}">
                    {pMeta.icon} {pMeta.label}
                  </span>
                </span>
                <span class="leader-playtime">{formatPlaytime(game.playtimeMinutes)}</span>
              </a>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Storage Snapshot Card -->
      <div class="storage-snap-card">
        <div class="dash-section-head" style="margin-bottom:0">
          <div>
            <h2>Storage &amp; Drive Overview</h2>
            <p class="muted-sm">Disk utilization across physical and logical drives.</p>
          </div>
          <a href="/storage" class="section-link">Detailed Analytics →</a>
        </div>

        <div class="storage-snap-stats">
          <div class="storage-stat-box">
            <span class="stat-label">Games Footprint</span>
            <span class="stat-val">{formatBytes(totalStorageBytes)}</span>
          </div>
          <div class="storage-stat-box">
            <span class="stat-label">Total Free Space</span>
            <span class="stat-val">{formatBytes(totalFreeDiskSpace)}</span>
          </div>
        </div>

        {#if disks.length > 0}
          <div class="snap-drive-list">
            {#each disks.slice(0, 3) as disk (disk.mount)}
              {@const used = disk.total - disk.free}
              {@const pct = ((used / (disk.total || 1)) * 100).toFixed(0)}
              <div class="snap-drive-row">
                <div class="drive-head">
                  <span><strong>{disk.mount}</strong> ({disk.name?.trim() || "Local Disk"})</span>
                  <span>{formatBytes(disk.free)} free of {formatBytes(disk.total)} ({pct}% used)</span>
                </div>
                <div class="drive-track">
                  <span style="width:{pct}%"></span>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
