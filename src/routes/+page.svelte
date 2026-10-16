<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { customNames, error, games, install, loading, settings } from "$lib/stores";
  import type { DiskInfo, DownloadItem, FreeGamePromotion, Game, PatchNote, Platform, StorageReport, StoreDeal } from "$lib/types";
  import "$styles/dashboard.scss";

  let storageReport = $state<StorageReport | null>(null);
  let launchingAppId = $state<number | null>(null);
  let launchMsg = $state<string | null>(null);

  type DashSubPage = "none" | "notifications" | "downloads";
  type NotificationTab = "free-games" | "deals" | "patch-notes";
  let activeSubPage = $state<DashSubPage>("none");
  let notificationTab = $state<NotificationTab>("free-games");

  let freeGamePromotions = $state<FreeGamePromotion[]>([]);
  let storeDeals = $state<StoreDeal[]>([]);
  let patchNotes = $state<PatchNote[]>([]);
  let loadingNotifications = $state(false);
  let activeDownloads = $state<DownloadItem[]>([]);

  let activeFreeGames = $derived(freeGamePromotions.filter((p) => p.isActive));
  let upcomingFreeGames = $derived(freeGamePromotions.filter((p) => !p.isActive));
  let totalNotificationCount = $derived(
    activeFreeGames.length + (patchNotes.length > 0 ? Math.min(patchNotes.length, 5) : 0)
  );

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
    other: { label: "Other", icon: "⚙", color: "#94a3b8" }
  };

  onMount(async () => {
    await refresh();
  });

  async function loadNotifications() {
    try {
      loadingNotifications = true;
      const candidateGames: [string, string][] = $games
        .filter((g) => g.installed)
        .slice(0, 10)
        .map((g) => [g.appId.toString(), g.name]);

      const [promos, deals, notes] = await Promise.all([
        api.getFreeGamePromotions().catch(() => []),
        api.getStoreDeals().catch(() => []),
        api.getLibraryPatchNotes(candidateGames.length > 0 ? candidateGames : undefined).catch(() => [])
      ]);

      freeGamePromotions = promos;
      storeDeals = deals;
      patchNotes = notes;
    } catch (e) {
      console.warn("Could not load notifications:", e);
    } finally {
      loadingNotifications = false;
    }
  }

  async function loadDownloads() {
    try {
      const [launchers, eos] = await Promise.all([
        api.getInstalledLaunchers().catch(() => []),
        api.getEosOverlayStatus().catch(() => null)
      ]);
      const list: DownloadItem[] = [];
      const epic = launchers.find((l) => l.id === "epic");
      if (epic?.installed) {
        list.push({
          id: "legendary-cli",
          name: "Legendary CLI (Native Epic Games Backend)",
          category: "tool",
          progress: 100,
          status: "completed",
        });
      }
      if (eos?.isInstalled) {
        list.push({
          id: "eos-overlay",
          name: `Epic Online Services Overlay ${eos.version ? `(${eos.version})` : ""}`,
          category: "overlay",
          progress: 100,
          status: "completed",
        });
      }
      activeDownloads = list;
    } catch (e) {
      console.warn("Could not query download items:", e);
    }
  }

  async function handleOpenUrl(url: string) {
    if (!url) return;
    try {
      await api.openExternalUrl(url);
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  async function handleOpenToolsFolder() {
    try {
      await api.openToolsFolder();
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  async function refresh(isManual = false) {
    const hasData = $games.length > 0;
    if (!hasData || isManual) {
      loading.set(true);
    }
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
      if (!hasData || isManual) {
        loading.set(false);
      }
    }

    // Load background notification feeds and downloads asynchronously without blocking dashboard mount
    loadNotifications().catch(() => {});
    loadDownloads().catch(() => {});
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
      <button
        class="btn btn-ghost"
        class:btn-accent={activeSubPage === "notifications"}
        onclick={() => (activeSubPage = activeSubPage === "notifications" ? "none" : "notifications")}
        title="Store giveaways, deals, and game update notes"
      >
        <span>🔔</span>
        <span>Notifications</span>
        {#if totalNotificationCount > 0}
          <span class="pill pill-accent" style="font-size:0.62rem;padding:0.05rem 0.35rem">
            {totalNotificationCount}
          </span>
        {/if}
      </button>
      <button
        class="btn btn-ghost"
        class:btn-accent={activeSubPage === "downloads"}
        onclick={() => (activeSubPage = activeSubPage === "downloads" ? "none" : "downloads")}
        title="View active tool downloads and component status"
      >
        <span>📥</span>
        <span>Downloads</span>
        {#if activeDownloads.length > 0}
          <span class="pill pill-ok" style="font-size:0.62rem;padding:0.05rem 0.35rem">
            {activeDownloads.length}
          </span>
        {/if}
      </button>
      <button class="btn btn-ghost" onclick={() => refresh(true)} disabled={$loading}>
        <span class:spin={$loading}>🔄</span>
        <span>{$loading ? $t("common.refreshing") : $t("common.refresh")}</span>
      </button>
      <a href="/library" class="btn btn-primary">
        Browse Library →
      </a>
    </div>
  </div>

  <!-- Multi-Tab Notifications & Alerts Subpage Panel -->
  {#if activeSubPage === "notifications"}
    <section class="glass card" style="margin-bottom:1.5rem;padding:1.25rem;border-color:rgba(56,189,248,0.3);background:rgba(20,24,36,0.9);animation:fadeIn 0.2s ease">
      <div class="row-between" style="align-items:center;margin-bottom:0.75rem;flex-wrap:wrap;gap:0.75rem">
        <div>
          <h2 style="margin:0;font-size:1.15rem;display:flex;align-items:center;gap:0.4rem">
            <span>🔔</span> Store Alerts &amp; Library Updates
          </h2>
          <p class="muted-sm" style="margin:0.2rem 0 0">
            Unified giveaways, deals, and game update notes across your enabled platforms and installed library.
          </p>
        </div>
        <div class="row" style="gap:0.5rem;align-items:center">
          <button class="btn mini btn-ghost" onclick={loadNotifications} disabled={loadingNotifications}>
            {loadingNotifications ? "Checking…" : "↻ Refresh"}
          </button>
          <button class="btn mini btn-ghost" onclick={() => (activeSubPage = "none")}>
            ✕ Close
          </button>
        </div>
      </div>

      <!-- Navigation Tabs: Free Games, Deals & Promotions, Patch Notes -->
      <div class="row" style="gap:0.5rem;border-bottom:1px solid rgba(255,255,255,0.08);padding-bottom:0.75rem;margin-bottom:1rem;flex-wrap:wrap">
        <button
          class="btn mini"
          class:btn-primary={notificationTab === "free-games"}
          class:btn-ghost={notificationTab !== "free-games"}
          onclick={() => (notificationTab = "free-games")}
        >
          <span>🎁 Free Games</span>
          {#if activeFreeGames.length > 0}
            <span class="pill pill-accent" style="font-size:0.6rem;padding:0 0.35rem">
              {activeFreeGames.length}
            </span>
          {/if}
        </button>
        <button
          class="btn mini"
          class:btn-primary={notificationTab === "deals"}
          class:btn-ghost={notificationTab !== "deals"}
          onclick={() => (notificationTab = "deals")}
        >
          <span>🏷️ Deals &amp; Promotions</span>
          {#if storeDeals.length > 0}
            <span class="pill pill-dim" style="font-size:0.6rem;padding:0 0.35rem">
              {storeDeals.length}
            </span>
          {/if}
        </button>
        <button
          class="btn mini"
          class:btn-primary={notificationTab === "patch-notes"}
          class:btn-ghost={notificationTab !== "patch-notes"}
          onclick={() => (notificationTab = "patch-notes")}
        >
          <span>📰 Patch Notes</span>
          {#if patchNotes.length > 0}
            <span class="pill pill-ok" style="font-size:0.6rem;padding:0 0.35rem">
              {patchNotes.length}
            </span>
          {/if}
        </button>
      </div>

      {#if loadingNotifications}
        <p class="muted-sm" style="padding:1.5rem 0;text-align:center">Checking live storefronts and library updates…</p>

      <!-- TAB 1: FREE GAMES -->
      {:else if notificationTab === "free-games"}
        {#if freeGamePromotions.length === 0}
          <div style="padding:1.5rem;text-align:center" class="muted-sm">
            No free game giveaways detected right now on enabled platforms. Check back soon!
          </div>
        {:else}
          <!-- Active 100% Free Games -->
          {#if activeFreeGames.length > 0}
            <div style="margin-bottom:1.5rem">
              <h3 style="font-size:0.95rem;margin:0 0 0.75rem;color:var(--text);display:flex;align-items:center;gap:0.4rem">
                <span>⚡</span> Active Now (100% Free to Keep)
              </h3>
              <div style="display:grid;grid-template-columns:repeat(auto-fill, minmax(280px, 1fr));gap:1rem">
                {#each activeFreeGames as promo (promo.id)}
                  {@const pMeta = PLATFORM_META[promo.platform as Platform] || PLATFORM_META.epic}
                  <div class="glass" style="display:flex;flex-direction:column;border-radius:8px;overflow:hidden;border:1px solid rgba(255,255,255,0.08);background:rgba(0,0,0,0.3)">
                    {#if promo.imageUrl}
                      <div style="height:140px;overflow:hidden;position:relative">
                        <img src={promo.imageUrl} alt={promo.title} style="width:100%;height:100%;object-fit:cover" />
                        <span class="pill pill-accent" style="position:absolute;top:8px;right:8px;font-size:0.65rem">
                          100% Free
                        </span>
                        {#if promo.worth}
                          <span class="pill pill-dim" style="position:absolute;bottom:8px;left:8px;font-size:0.62rem">
                            Worth {promo.worth}
                          </span>
                        {/if}
                      </div>
                    {/if}
                    <div style="padding:0.85rem;display:flex;flex-direction:column;gap:0.35rem;flex:1">
                      <div class="row-between" style="align-items:center">
                        <strong style="font-size:0.95rem;color:var(--text)">{promo.title}</strong>
                        <span class="pill pill-dim" style="font-size:0.62rem;border-color:{pMeta.color}40;color:{pMeta.color}">
                          {pMeta.icon} {pMeta.label}
                        </span>
                      </div>
                      {#if promo.description}
                        <p class="muted-xs" style="margin:0;line-height:1.35;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden">
                          {promo.description}
                        </p>
                      {/if}
                      <div class="row-between" style="align-items:center;margin-top:auto;padding-top:0.6rem">
                        <span class="muted-xs" style="font-size:0.7rem">
                          {#if promo.endDate}
                            Until {new Date(promo.endDate).toLocaleDateString()}
                          {:else}
                            Active Now
                          {/if}
                        </span>
                        <button
                          class="btn mini btn-accent"
                          onclick={() => handleOpenUrl(promo.url)}
                          title="Claim {promo.title} in default browser"
                        >
                          Claim on {pMeta.label} ↗
                        </button>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Upcoming Free Giveaways -->
          {#if upcomingFreeGames.length > 0}
            <div>
              <h3 style="font-size:0.95rem;margin:0 0 0.75rem;color:var(--text-muted);display:flex;align-items:center;gap:0.4rem">
                <span>⏳</span> Upcoming Giveaways (Coming Soon)
              </h3>
              <div style="display:grid;grid-template-columns:repeat(auto-fill, minmax(280px, 1fr));gap:1rem">
                {#each upcomingFreeGames as promo (promo.id)}
                  {@const pMeta = PLATFORM_META[promo.platform as Platform] || PLATFORM_META.epic}
                  <div class="glass" style="display:flex;flex-direction:column;border-radius:8px;overflow:hidden;border:1px solid rgba(255,255,255,0.08);background:rgba(0,0,0,0.2);opacity:0.88">
                    {#if promo.imageUrl}
                      <div style="height:140px;overflow:hidden;position:relative">
                        <img src={promo.imageUrl} alt={promo.title} style="width:100%;height:100%;object-fit:cover" />
                        <span class="pill pill-dim" style="position:absolute;top:8px;right:8px;font-size:0.65rem">
                          Coming Soon
                        </span>
                      </div>
                    {/if}
                    <div style="padding:0.85rem;display:flex;flex-direction:column;gap:0.35rem;flex:1">
                      <div class="row-between" style="align-items:center">
                        <strong style="font-size:0.95rem;color:var(--text)">{promo.title}</strong>
                        <span class="pill pill-dim" style="font-size:0.62rem;border-color:{pMeta.color}40;color:{pMeta.color}">
                          {pMeta.icon} {pMeta.label}
                        </span>
                      </div>
                      {#if promo.description}
                        <p class="muted-xs" style="margin:0;line-height:1.35;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden">
                          {promo.description}
                        </p>
                      {/if}
                      <div class="row-between" style="align-items:center;margin-top:auto;padding-top:0.6rem">
                        <span class="muted-xs" style="font-size:0.7rem;color:var(--accent)">
                          {#if promo.startDate}
                            Free starting {new Date(promo.startDate).toLocaleDateString()}
                          {:else}
                            Coming Soon
                          {/if}
                        </span>
                        <button
                          class="btn mini btn-ghost"
                          onclick={() => handleOpenUrl(promo.url)}
                          title="Preview {promo.title} on {pMeta.label}"
                        >
                          Preview on {pMeta.label} ↗
                        </button>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {/if}

      <!-- TAB 2: DEALS & PROMOTIONS -->
      {:else if notificationTab === "deals"}
        {#if storeDeals.length === 0}
          <div style="padding:1.5rem;text-align:center" class="muted-sm">
            No featured store promotions detected right now on enabled platforms. Check back soon!
          </div>
        {:else}
          <div style="display:grid;grid-template-columns:repeat(auto-fill, minmax(280px, 1fr));gap:1rem">
            {#each storeDeals as deal (deal.id)}
              {@const pMeta = PLATFORM_META[deal.platform as Platform] || PLATFORM_META.steam}
              <div class="glass" style="display:flex;flex-direction:column;border-radius:8px;overflow:hidden;border:1px solid rgba(255,255,255,0.08);background:rgba(0,0,0,0.3)">
                {#if deal.imageUrl}
                  <div style="height:140px;overflow:hidden;position:relative">
                    <img src={deal.imageUrl} alt={deal.title} style="width:100%;height:100%;object-fit:cover" />
                    <span class="pill pill-accent" style="position:absolute;top:8px;right:8px;font-size:0.68rem;font-weight:700">
                      -{deal.discountPercent}%
                    </span>
                  </div>
                {/if}
                <div style="padding:0.85rem;display:flex;flex-direction:column;gap:0.35rem;flex:1">
                  <div class="row-between" style="align-items:center">
                    <strong style="font-size:0.95rem;color:var(--text)">{deal.title}</strong>
                    <span class="pill pill-dim" style="font-size:0.62rem;border-color:{pMeta.color}40;color:{pMeta.color}">
                      {pMeta.icon} {pMeta.label}
                    </span>
                  </div>
                  <div class="row-between" style="align-items:center;margin-top:auto;padding-top:0.6rem">
                    <div style="display:flex;align-items:baseline;gap:0.4rem">
                      {#if deal.originalPrice}
                        <span class="muted-xs" style="text-decoration:line-through;font-size:0.75rem">{deal.originalPrice}</span>
                      {/if}
                      <strong style="color:var(--accent);font-size:0.98rem">{deal.finalPrice}</strong>
                    </div>
                    <button
                      class="btn mini btn-primary"
                      onclick={() => handleOpenUrl(deal.url)}
                      title="Open {deal.title} deal in browser"
                    >
                      View Deal ↗
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

      <!-- TAB 3: PATCH NOTES & UPDATES -->
      {:else if notificationTab === "patch-notes"}
        {#if patchNotes.length === 0}
          <div style="padding:1.5rem;text-align:center" class="muted-sm">
            No recent patch notes detected for installed library games.
          </div>
        {:else}
          <div style="display:grid;grid-template-columns:repeat(auto-fill, minmax(320px, 1fr));gap:1rem">
            {#each patchNotes as note (note.id)}
              {@const pMeta = PLATFORM_META[note.platform as Platform] || PLATFORM_META.steam}
              <div class="glass" style="display:flex;flex-direction:column;padding:0.9rem;border-radius:8px;border:1px solid rgba(255,255,255,0.08);background:rgba(0,0,0,0.3)">
                <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
                  <span class="pill pill-dim" style="font-size:0.65rem;border-color:{pMeta.color}40;color:{pMeta.color}">
                    {pMeta.icon} {note.gameName}
                  </span>
                  {#if note.date}
                    <span class="muted-xs" style="font-size:0.68rem">
                      {new Date(note.date).toLocaleDateString()}
                    </span>
                  {/if}
                </div>
                <strong style="font-size:0.95rem;color:var(--text);margin-bottom:0.35rem;line-height:1.3">
                  {note.title}
                </strong>
                {#if note.summary}
                  <p class="muted-xs" style="margin:0 0 0.75rem;line-height:1.4;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden">
                    {note.summary}
                  </p>
                {/if}
                <div class="row-between" style="align-items:center;margin-top:auto;padding-top:0.4rem">
                  <span class="muted-xs" style="font-size:0.65rem">
                    {note.author ? `By ${note.author}` : "Official Announcement"}
                  </span>
                  <button
                    class="btn mini btn-ghost"
                    onclick={() => handleOpenUrl(note.url)}
                    title="Open official update announcement in browser"
                  >
                    Read Notes ↗
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </section>
  {/if}

  <!-- Active Downloads Subpage Panel -->
  {#if activeSubPage === "downloads"}
    <section class="glass card" style="margin-bottom:1.5rem;padding:1.25rem;border-color:rgba(56,189,248,0.3);background:rgba(20,24,36,0.85);animation:fadeIn 0.2s ease">
      <div class="row-between" style="align-items:center;margin-bottom:0.75rem">
        <div>
          <h2 style="margin:0;font-size:1.15rem;display:flex;align-items:center;gap:0.4rem">
            <span>📥</span> Active Downloads &amp; Component Status
          </h2>
          <p class="muted-sm" style="margin:0.2rem 0 0">
            Real-time status of runtime components, CLI tools, overlays, and background tasks.
          </p>
        </div>
        <div class="row" style="gap:0.5rem;align-items:center">
          <button class="btn mini btn-ghost" onclick={handleOpenToolsFolder} title="Open managed tools directory in Explorer">
            Tools Folder ↗
          </button>
          <button class="btn mini btn-ghost" onclick={() => (activeSubPage = "none")}>
            ✕ Close
          </button>
        </div>
      </div>

      <div style="display:flex;flex-direction:column;gap:0.5rem;margin-top:0.5rem">
        {#each activeDownloads as dl (dl.id)}
          <div class="glass" style="display:flex;align-items:center;justify-content:space-between;padding:0.75rem 1rem;border-radius:6px;border:1px solid rgba(255,255,255,0.06)">
            <div style="display:flex;align-items:center;gap:0.65rem">
              <span style="font-size:1.1rem">
                {dl.category === "tool" ? "⚡" : dl.category === "overlay" ? "🎮" : "📦"}
              </span>
              <div>
                <strong style="font-size:0.88rem;color:var(--text)">{dl.name}</strong>
                <span class="muted-xs" style="display:block">Category: {dl.category}</span>
              </div>
            </div>
            <div class="row" style="align-items:center;gap:0.5rem">
              <span class="pill pill-ok" style="font-size:0.65rem">
                ✓ {dl.status === "completed" ? "Ready & Installed" : dl.status}
              </span>
            </div>
          </div>
        {/each}
        {#if activeDownloads.length === 0}
          <div class="muted-sm" style="padding:1rem;text-align:center">
            No active downloads. All components and background tasks are idle.
          </div>
        {/if}
      </div>
    </section>
  {/if}

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
