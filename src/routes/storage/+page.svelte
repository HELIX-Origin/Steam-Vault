<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error, games, settings, storageReport } from "$lib/stores";
  import PlatformIcon from "$lib/components/PlatformIcon.svelte";
  import type { DiskInfo, Game, GamePlacement, Platform, StorageReport } from "$lib/types";
  import "$styles/storage.scss";

  let report = $derived($storageReport);
  let libraryGames = $derived($games);
  let busy = $state(false);

  // Filters & Search
  let search = $state("");
  let selectedPlatform = $state<Platform | "all">("all");
  let sortBy = $state<"size-desc" | "size-asc" | "name-asc" | "name-desc">("size-desc");

  interface PlatformMeta {
    id: Platform;
    label: string;
    icon: string;
    color: string;
  }

  const PLATFORMS: PlatformMeta[] = [
    { id: "steam", label: "Steam", icon: "♨", color: "#38bdf8" },
    { id: "gog", label: "GOG Galaxy", icon: "◈", color: "#a855f7" },
    { id: "epic", label: "Epic Games", icon: "▲", color: "#facc15" },
    { id: "xbox", label: "Xbox / MS Store", icon: "⨂", color: "#22c55e" },
    { id: "ea", label: "EA App", icon: "●", color: "#f97316" },
    { id: "ubisoft", label: "Ubisoft Connect", icon: "🌀", color: "#06b6d4" },
    { id: "battlenet", label: "Battle.net", icon: "❄", color: "#3b82f6" },
    { id: "itch", label: "itch.io", icon: "⌗", color: "#ef4444" },
    { id: "amazon", label: "Amazon Games", icon: "📦", color: "#eab308" },
    { id: "rockstar", label: "Rockstar Games", icon: "★", color: "#fbbf24" },
    { id: "other", label: "Other / Standalone", icon: "⚙", color: "#94a3b8" }
  ];

  function isPluginEnabled(pluginId: string): boolean {
    if ($settings?.enabledPlugins && $settings.enabledPlugins[pluginId] !== undefined) {
      return $settings.enabledPlugins[pluginId];
    }
    return true; // default to enabled if not explicitly disabled
  }

  function isPlatformPluginEnabled(platformId: Platform): boolean {
    const pluginId = platformId === "other" ? "platform-standalone" : `platform-${platformId}`;
    return isPluginEnabled(pluginId);
  }

  let activePlatforms = $derived(
    PLATFORMS.filter((p) => isPlatformPluginEnabled(p.id))
  );

  onMount(() => {
    // Non-blocking background worker: UI renders immediately, values populate in background
    refresh();
  });

  async function refresh(isManual = false) {
    busy = true;
    try {
      const storageEnabled = isPluginEnabled("utility-storage");
      const [storageRep, lib] = await Promise.all([
        storageEnabled ? api.getStorageReport().catch(() => null) : Promise.resolve(null),
        $games.length > 0 ? Promise.resolve($games) : api.loadLibrary().catch(() => [])
      ]);
      storageReport.set(storageRep);
      if (lib && lib.length > 0) {
        games.set(lib);
      }
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  interface UnifiedInstalledGame {
    appId: number;
    name: string;
    platform: Platform;
    sizeOnDisk: number;
    installDir: string | null;
    libraryPath: string | null;
    installed: boolean;
  }

  // Deduplicate and combine all installed games across Steam and external platforms
  let allInstalledGames = $derived.by<UnifiedInstalledGame[]>(() => {
    const map = new Map<string, UnifiedInstalledGame>();

    // 1. Ingest report.games
    if (report?.games) {
      for (const g of report.games) {
        if (!g.installed && g.sizeOnDisk === 0) continue;
        const plat = (g.platform as Platform) || "steam";
        if (!isPlatformPluginEnabled(plat)) continue;
        const key = `${plat}:${g.appId}`;
        map.set(key, {
          appId: g.appId,
          name: g.name,
          platform: plat,
          sizeOnDisk: g.sizeOnDisk || 0,
          installDir: g.installDir,
          libraryPath: g.libraryPath,
          installed: g.installed
        });
      }
    }

    // 2. Ingest libraryGames (e.g. non-Steam standalone titles)
    for (const g of libraryGames) {
      if (!g.installed && !g.sizeOnDisk) continue;
      if (!isPlatformPluginEnabled(g.platform)) continue;
      const key = `${g.platform}:${g.appId}`;
      if (!map.has(key)) {
        map.set(key, {
          appId: g.appId,
          name: g.name,
          platform: g.platform,
          sizeOnDisk: g.sizeOnDisk || 0,
          installDir: g.installDir,
          libraryPath: g.libraryPath,
          installed: g.installed
        });
      }
    }

    return Array.from(map.values()).sort((a, b) => b.sizeOnDisk - a.sizeOnDisk);
  });

  let totalGamingBytes = $derived(
    allInstalledGames.reduce((acc, g) => acc + g.sizeOnDisk, 0)
  );

  let disks = $derived<DiskInfo[]>(report?.disks ?? []);
  let totalFreeDiskSpace = $derived(disks.reduce((sum, d) => sum + d.free, 0));
  let totalDiskCapacity = $derived(disks.reduce((sum, d) => sum + d.total, 0));

  // Platform breakdown statistics
  interface PlatformStatSummary {
    meta: PlatformMeta;
    bytes: number;
    gameCount: number;
    percentage: number;
  }

  let platformStats = $derived.by<PlatformStatSummary[]>(() => {
    return activePlatforms
      .map((meta) => {
        const platformGames = allInstalledGames.filter((g) => g.platform === meta.id);
        const bytes = platformGames.reduce((sum, g) => sum + g.sizeOnDisk, 0);
        const percentage = totalGamingBytes > 0 ? (bytes / totalGamingBytes) * 100 : 0;
        return {
          meta,
          bytes,
          gameCount: platformGames.length,
          percentage
        };
      })
      .filter((stat) => stat.gameCount > 0);
  });

  // Calculate gaming footprint per disk
  function getDiskGamingStats(disk: DiskInfo): { gamingBytes: number; gameCount: number; pct: number } {
    const mountLower = (disk.mount || "").toLowerCase().replace(/\\+$/, "").replace(/\/+$/, "");
    const matchingGames = allInstalledGames.filter((g) => {
      const path = (g.installDir || g.libraryPath || "").toLowerCase();
      return path.startsWith(mountLower);
    });
    const gamingBytes = matchingGames.reduce((sum, g) => sum + g.sizeOnDisk, 0);
    const pct = disk.total > 0 ? (gamingBytes / disk.total) * 100 : 0;
    return {
      gamingBytes,
      gameCount: matchingGames.length,
      pct
    };
  }

  // Filtered games for the Storage Explorer
  let filteredGames = $derived.by<UnifiedInstalledGame[]>(() => {
    const q = search.trim().toLowerCase();
    return allInstalledGames
      .filter((g) => {
        if (selectedPlatform !== "all" && g.platform !== selectedPlatform) return false;
        if (q && !g.name.toLowerCase().includes(q)) return false;
        return true;
      })
      .sort((a, b) => {
        switch (sortBy) {
          case "size-desc":
            return b.sizeOnDisk - a.sizeOnDisk;
          case "size-asc":
            return a.sizeOnDisk - b.sizeOnDisk;
          case "name-asc":
            return a.name.localeCompare(b.name);
          case "name-desc":
            return b.name.localeCompare(a.name);
          default:
            return 0;
        }
      });
  });

  let maxGameSize = $derived(
    filteredGames.length > 0 ? Math.max(...filteredGames.map((g) => g.sizeOnDisk), 1) : 1
  );

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

  function getPlatformMeta(p: Platform): PlatformMeta {
    return PLATFORMS.find((item) => item.id === p) || {
      id: "other",
      label: "Other",
      icon: "⚙",
      color: "#94a3b8"
    };
  }
</script>

<div class="row-between" style="margin-bottom:1.25rem">
  <div>
    <div class="row" style="align-items:center;gap:0.75rem">
      <h1 style="margin:0">{$t("storage.title")}</h1>
      {#if busy}
        <span class="pill pill-accent2" style="font-size:0.72rem;letter-spacing:0.03em;text-transform:uppercase">
          <span class="spin" style="display:inline-block;margin-right:0.25rem">↻</span> Background Sync
        </span>
      {/if}
    </div>
    <p class="muted" style="margin:0.25rem 0 0 0">Storage utilization, drive footprints, and installed games across all supported platforms.</p>
  </div>
  <div class="row" style="gap:0.5rem">
    <button class="btn btn-ghost" onclick={() => refresh(true)} disabled={busy}>
      <span class:spin={busy}>🔄</span>
      <span>{busy ? $t("common.refreshing") : $t("common.refresh")}</span>
    </button>
  </div>
</div>

  <!-- Storage KPI Overview Cards -->
  <div class="grid kpi-grid">
    <div class="glass card kpi-card">
      <span class="kpi-num">{formatBytes(totalGamingBytes)}</span>
      <span class="kpi-label">Total Games Footprint</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{formatBytes(totalFreeDiskSpace)}</span>
      <span class="kpi-label">Free Across {disks.length} Drives</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{allInstalledGames.length}</span>
      <span class="kpi-label">Installed Games Tracked</span>
    </div>
    <div class="glass card kpi-card">
      <span class="kpi-num">{platformStats.length}</span>
      <span class="kpi-label">Active Enabled Platforms</span>
    </div>
  </div>

  <!-- Platform Storage Usage Statistics Section -->
  <div class="section-container">
    <div class="section-head">
      <div>
        <h2 class="section-title">Platform Storage Statistics</h2>
        <p class="muted-sm">Disk space and installation footprints across all supported gaming storefronts.</p>
      </div>
      <span class="muted-sm">{formatBytes(totalGamingBytes)} across {allInstalledGames.length} titles</span>
    </div>

    <div class="platforms-storage-grid">
      {#each platformStats as stat (stat.meta.id)}
        <div class="platform-storage-card" class:is-empty={stat.gameCount === 0}>
          <div class="platform-head">
            <div class="platform-meta">
              <span class="platform-icon-wrap" style="color:{stat.meta.color}">
                <PlatformIcon platform={stat.meta.id} size={16} />
              </span>
              <span class="platform-name">{stat.meta.label}</span>
            </div>
            <span class="platform-count-pill">
              {stat.gameCount} {stat.gameCount === 1 ? "game" : "games"}
            </span>
          </div>

          <div class="platform-stats-row">
            <span class="platform-bytes">{formatBytes(stat.bytes)}</span>
            <span class="platform-pct">{stat.percentage.toFixed(1)}% of total</span>
          </div>

          <div class="platform-bar-track">
            <span
              class="platform-bar-fill"
              style="width:{stat.percentage}%;background:{stat.meta.color}"
            ></span>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Connected Storage Drives Section -->
  <div class="section-container">
    <div class="section-head">
      <div>
        <h2 class="section-title">Connected Storage Drives</h2>
        <p class="muted-sm">Physical and logical storage partitions with game footprints per volume.</p>
      </div>
      <span class="muted-sm">{disks.length} drive{disks.length === 1 ? "" : "s"} detected</span>
    </div>

    <div class="grid drives">
      {#if disks.length > 0}
        {#each disks as disk (disk.mount)}
          {@const driveStats = getDiskGamingStats(disk)}
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
            {#if driveStats.gameCount > 0}
              <div class="drive-game-meta">
                <span>Games: <strong>{formatBytes(driveStats.gamingBytes)}</strong> ({driveStats.gameCount} titles)</span>
                <span>{driveStats.pct.toFixed(1)}% of disk</span>
              </div>
            {/if}
          </div>
        {/each}
      {:else}
        <div class="glass card drive-card" style="grid-column: 1 / -1; padding: 1.5rem; text-align: center;">
          <p class="muted-sm" style="margin: 0;">
            {#if !isPluginEnabled("utility-storage")}
              The <strong>Storage &amp; Drive Analyzer</strong> feature is currently disabled in <a href="/settings" style="color:var(--accent)">Settings</a>.
            {:else if busy}
              Analyzing drive capacities in background…
            {:else}
              No physical drives detected.
            {/if}
          </p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Installed Games Storage Breakdown Section -->
  <div class="section-head">
    <div>
      <h2 class="section-title">Installed Games Storage Explorer</h2>
      <p class="muted-sm">Installed titles ranked by disk space usage across all storefront platforms.</p>
    </div>
    <span class="muted-sm">{filteredGames.length} matching titles</span>
  </div>

  <div class="section-container">
    <div class="storage-explorer-card">
      <div class="storage-toolbar">
        <div class="toolbar-controls">
          <div class="search-input-wrap">
            <input
              type="text"
              bind:value={search}
              placeholder="Search installed titles…"
            />
            {#if search}
              <button class="clear-btn" onclick={() => (search = "")}>✕</button>
            {/if}
          </div>

          <div class="row" style="gap:0.5rem;align-items:center">
            <span class="muted-sm">Sort:</span>
            <select bind:value={sortBy} class="btn mini" style="background:var(--bg-1);border-color:var(--border)">
              <option value="size-desc">Largest Size First</option>
              <option value="size-asc">Smallest Size First</option>
              <option value="name-asc">Name (A → Z)</option>
              <option value="name-desc">Name (Z → A)</option>
            </select>
          </div>
        </div>

        <!-- Platform Filter Pills -->
        <div class="platform-pills-row">
          <button
            class="filter-pill"
            class:active={selectedPlatform === "all"}
            onclick={() => (selectedPlatform = "all")}
          >
            All Platforms ({allInstalledGames.length})
          </button>
          {#each activePlatforms as p}
            {@const count = allInstalledGames.filter((g) => g.platform === p.id).length}
            {#if count > 0}
              <button
                class="filter-pill"
                class:active={selectedPlatform === p.id}
                onclick={() => (selectedPlatform = p.id)}
              >
                <PlatformIcon platform={p.id} size={12} color={p.color} /> {p.label} ({count})
              </button>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Games Storage Table / List -->
      {#if filteredGames.length > 0}
        <div class="games-storage-list">
          {#each filteredGames as game, index (game.platform + "-" + game.appId)}
            {@const pMeta = getPlatformMeta(game.platform)}
            <div class="game-storage-row">
              <span class="row-rank">#{index + 1}</span>

              <div class="row-main">
                <span class="row-title" title={game.name}>{game.name}</span>
                {#if game.installDir || game.libraryPath}
                  <span class="row-path" title={game.installDir || game.libraryPath}>
                    {game.installDir || game.libraryPath}
                  </span>
                {/if}
              </div>

              <div class="row-platform">
                <span class="pill pill-dim" style="border-color:{pMeta.color}40;color:{pMeta.color}">
                  <PlatformIcon platform={pMeta.id} size={12} /> {pMeta.label}
                </span>
              </div>

              <div class="row-size">
                <span class="row-size-num">{formatBytes(game.sizeOnDisk)}</span>
                <div class="row-size-bar">
                  <span style="width:{(game.sizeOnDisk / maxGameSize) * 100}%"></span>
                </div>
              </div>

              <div class="row-action">
                <a href={`/library?game=${game.appId}`} class="inspect-link">
                  Manage in Library →
                </a>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="empty-storage-msg">
          <p>No installed games match the active filters or search query.</p>
        </div>
      {/if}
    </div>
  </div>
