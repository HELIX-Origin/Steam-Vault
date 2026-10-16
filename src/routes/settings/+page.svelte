<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { STEAM_LANGUAGES, setLocale, syncWithSettingsLanguage, t } from "$lib/i18n";
  import { collections, error, games, install, settings, steamRunning } from "$lib/stores";
  import type { CacheStats, PluginInfo, Settings } from "$lib/types";
  import PlatformsSection from "$lib/components/settings/PlatformsSection.svelte";
  import "$styles/settings.scss";

  type SettingsTab = "general" | "platforms" | "plugins";
  let activeTab = $state<SettingsTab>("general");

  let form = $state<Settings | null>(null);
  let cacheStats = $state<CacheStats | null>(null);
  let syncingLibraryMeta = $state(false);
  let syncLibraryAutoArtwork = $state(true);
  let syncLibraryMsg = $state<string | null>(null);
  let clearingCache = $state(false);
  let clearCacheMsg = $state<string | null>(null);
  let showClearCacheConfirm = $state(false);
  let busy = $state(false);
  let saved = $state(false);

  // Community & Drop-In Plugins State
  let installedPlugins = $state<PluginInfo[]>([]);
  let loadingPlugins = $state(false);
  let pluginUrlInput = $state("");
  let installingPlugin = $state(false);
  let pluginActionMsg = $state<string | null>(null);
  let pluginFilterType = $state<string>("all");

  async function loadPlugins() {
    loadingPlugins = true;
    try {
      installedPlugins = await api.getInstalledPlugins();
    } catch (e) {
      console.error("Failed to load plugins:", e);
    } finally {
      loadingPlugins = false;
    }
  }

  async function handleTogglePlugin(plugin: PluginInfo) {
    try {
      await api.togglePlugin(plugin.id, !plugin.enabled);
      plugin.enabled = !plugin.enabled;
      installedPlugins = [...installedPlugins];
      pluginActionMsg = `${plugin.name} is now ${plugin.enabled ? "enabled" : "disabled"}.`;
      setTimeout(() => (pluginActionMsg = null), 3000);
    } catch (e) {
      pluginActionMsg = `Error: ${errorMessage(e)}`;
    }
  }

  async function handleInstallFromUrl() {
    if (!pluginUrlInput.trim()) return;
    installingPlugin = true;
    pluginActionMsg = "Fetching plugin manifest from GitHub/URL…";
    try {
      const installed = await api.installPluginFromUrl(pluginUrlInput.trim());
      pluginActionMsg = `Successfully installed "${installed.name}" v${installed.version}!`;
      pluginUrlInput = "";
      await loadPlugins();
      setTimeout(() => (pluginActionMsg = null), 4000);
    } catch (e) {
      pluginActionMsg = `Installation failed: ${errorMessage(e)}`;
    } finally {
      installingPlugin = false;
    }
  }

  async function handleOpenPluginsFolder() {
    try {
      await api.openPluginsFolder();
    } catch (e) {
      pluginActionMsg = `Could not open folder: ${errorMessage(e)}`;
    }
  }

  async function refreshAllMetadata() {
    syncingLibraryMeta = true;
    syncLibraryMsg = null;
    try {
      const res = await api.syncLibraryMetadata(form?.accountId3 ?? undefined, true, syncLibraryAutoArtwork);
      syncLibraryMsg = `Updated ${res.fetched} games (${res.cached} cached, ${res.unavailable} unresolved).`;
      cacheStats = await api.getCacheStats().catch(() => null);
    } catch (e) {
      syncLibraryMsg = errorMessage(e);
    } finally {
      syncingLibraryMeta = false;
    }
  }

  async function handleClearCache() {
    clearingCache = true;
    clearCacheMsg = null;
    try {
      const res = await api.clearMetadataCache(form?.accountId3 ?? undefined, true);
      clearCacheMsg = `Cache reset: ${res.totalMetadataRecords} metadata records re-indexed. ${res.message}`;
      cacheStats = await api.getCacheStats().catch(() => null);
      showClearCacheConfirm = false;
    } catch (e) {
      clearCacheMsg = errorMessage(e);
    } finally {
      clearingCache = false;
    }
  }

  /** Non-Steam sources surfaced in Settings (mirrors the backend platform ids). */
  const platformList: { id: string; label: string }[] = [
    { id: "gog", label: "GOG" },
    { id: "epic", label: "Epic Games" },
    { id: "itch", label: "itch.io" },
    { id: "ubisoft", label: "Ubisoft Connect" },
    { id: "ea", label: "EA" },
    { id: "battlenet", label: "Battle.net" },
    { id: "amazon", label: "Amazon Games" },
    { id: "rockstar", label: "Rockstar" },
    { id: "xbox", label: "Xbox / Microsoft Store" },
    { id: "other", label: "Other studios" }
  ];

  type SourceRow = { id: string; label: string; enabled: boolean; roots: string[] };
  let sources = $state<SourceRow[]>([]);
  let finding = $state<string | null>(null);

  /** Project saved preferences onto the fixed platform list. */
  function buildSources() {
    const savedById = new Map(
      (form?.platformSources ?? []).map((source) => [source.id, source])
    );
    sources = platformList.map((platform) => {
      const entry = savedById.get(platform.id);
      return {
        id: platform.id,
        label: platform.label,
        enabled: entry?.enabled ?? true,
        roots: [...(entry?.roots ?? [])]
      };
    });
  }

  /** Add another (empty) library folder input for a platform. */
  function addRoot(source: SourceRow) {
    source.roots = [...source.roots, ""];
    sources = [...sources];
  }

  /** Remove one library folder input. */
  function removeRoot(source: SourceRow, index: number) {
    source.roots = source.roots.filter((_, i) => i !== index);
    sources = [...sources];
  }

  /** Update one library folder input. */
  function setRoot(source: SourceRow, index: number, value: string) {
    source.roots = source.roots.map((root, i) => (i === index ? value : root));
    sources = [...sources];
  }

  /** Autofill a platform's detected library folders. */
  async function findFolders(source: SourceRow) {
    finding = source.id;
    try {
      const found = await api.findPlatformFolders(source.id);
      const roots = source.roots.filter((root) => root.trim());
      for (const folder of found) {
        if (!roots.includes(folder)) roots.push(folder);
      }
      source.roots = roots;
      source.enabled = true;
      sources = [...sources];
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      finding = null;
    }
  }

  function setTab(tab: SettingsTab) {
    activeTab = tab;
    const url = new URL(window.location.href);
    url.searchParams.set("tab", tab);
    window.history.replaceState({}, "", url.toString());
  }

  onMount(async () => {
    try {
      const params = new URLSearchParams(window.location.search);
      const tabParam = params.get("tab") as SettingsTab | null;
      if (tabParam && ["general", "platforms", "plugins"].includes(tabParam)) {
        activeTab = tabParam;
      }
      loadPlugins();

      const [loaded, detected, stats] = await Promise.all([
        api.getSettings(),
        api.detectSteam(),
        api.getCacheStats().catch(() => null)
      ]);
      cacheStats = stats;
      settings.set(loaded);
      form = { ...loaded };
      if (loaded.language) {
        syncWithSettingsLanguage(loaded.language);
      }
      install.set(detected);
      if (!form.steamRoot && detected) form.steamRoot = detected.root;
      if (!form.accountId3 && detected?.accounts[0]) form.accountId3 = detected.accounts[0].id3;
      buildSources();
    } catch (e) {
      error.set(errorMessage(e));
    }
  });

  async function detect() {
    try {
      const detected = await api.detectSteam();
      install.set(detected);
      if (detected && form) {
        form.steamRoot = detected.root;
        form.accountId3 ??= detected.accounts[0]?.id3 ?? null;
      }
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  async function save() {
    if (!form) return;
    form.platformSources = sources.map((source) => ({
      id: source.id,
      enabled: source.enabled,
      roots: source.roots.map((value) => value.trim()).filter(Boolean)
    }));
    busy = true;
    try {
      const updated = await api.saveSettings(form);
      settings.set(updated);
      form = { ...updated };
      if (updated.language) {
        setLocale(updated.language);
      }
      saved = true;
      setTimeout(() => (saved = false), 2200);
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="row-between" style="margin-bottom:1rem">
  <div>
    <h1>{$t("settings.title")}</h1>
    <p class="muted">{$t("settings.subtitle")}</p>
  </div>
  <div class="row" style="align-items:center;gap:0.75rem">
    {#if saved}<span class="pill pill-ok">{$t("common.saved")}</span>{/if}
    <button class="btn btn-primary" disabled={busy || !form} onclick={save}>
      {busy ? $t("common.saving") : $t("common.save")}
    </button>
  </div>
</div>

<!-- Section Navigation Tabs -->
<div class="settings-nav-tabs">
  <button class="tab-btn" class:active={activeTab === "general"} onclick={() => setTab("general")}>
    <span class="tab-icon">⚙️</span>
    <span>{$t("settings.title")}</span>
  </button>
  <button class="tab-btn" class:active={activeTab === "platforms"} onclick={() => setTab("platforms")}>
    <span class="tab-icon">⬡</span>
    <span>{$t("nav.platforms")}</span>
  </button>
  <button class="tab-btn" class:active={activeTab === "plugins"} onclick={() => { setTab("plugins"); loadPlugins(); }}>
    <span class="tab-icon">🧩</span>
    <span>Plugins</span>
  </button>
</div>

{#if form}
  {#if activeTab === "general"}
    <div class="settings-columns">
      <!-- Left Column -->
      <div class="settings-col">

        <!-- Steam Folder & Active Account -->
        <section class="glass-strong card">
          <div class="row-between" style="align-items:center;margin-bottom:0.75rem">
            <h2 class="section-title" style="margin:0">{$t("settings.steamFolder")}</h2>
            <button class="btn mini" onclick={detect}>{$t("settings.autoDetect")}</button>
          </div>
          <div class="field">
            <label for="root">Steam Installation Directory</label>
            <input id="root" bind:value={form.steamRoot} placeholder="C:\Program Files (x86)\Steam" />
          </div>
          <div class="field">
            <label for="account">{$t("settings.activeAccount")}</label>
            <select id="account" bind:value={form.accountId3}>
              <option value={null}>Automatic (first found)</option>
              {#each $install?.accounts ?? [] as account (account.id3)}
                <option value={account.id3}>
                  {(account.personaName ?? account.accountName) || "Unknown"} ({account.id3})
                </option>
              {/each}
            </select>
          </div>
          <div class="field" style="margin-top:0.5rem">
            <label for="cc">Store Country Code</label>
            <input id="cc" bind:value={form.countryCode} placeholder="us" />
          </div>
          {#if $install}
            <div class="kv" style="margin-top:0.5rem"><span class="muted">{$t("settings.librariesCount")}</span><span>{$install.libraries.length} folders</span></div>
            <div class="kv"><span class="muted">{$t("settings.accountsCount")}</span><span>{$install.accounts.length} accounts</span></div>
          {/if}
        </section>

        <!-- API Keys -->
        <section class="glass-strong card">
          <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
            <h2 class="section-title" style="margin:0">API &amp; Storefront Keys</h2>
            <span class="pill" style="font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em">Keyless Architecture</span>
          </div>
          <p class="muted-sm" style="margin-bottom:0.75rem">
            Universal Game Launcher uses direct native storefront integrations (Steam, GOG, Epic, Xbox, itch) and offline engine discovery. Third-party API keys are strictly optional.
          </p>
          <div class="field">
            <label for="steamkey">Steam Web API Key <span class="muted-xs">(Optional remote sync)</span></label>
            <input id="steamkey" type="password" bind:value={form.steamApiKey} placeholder="Optional — steamcommunity.com/dev/apikey" />
          </div>
          <div class="field">
            <label for="sgdb">SteamGridDB Key <span class="muted-xs">(Optional fallback)</span></label>
            <input id="sgdb" type="password" bind:value={form.steamgriddbApiKey} placeholder="SteamGridDB API key" />
          </div>
          <div class="row" style="gap:1rem">
            <div class="field" style="flex:1">
              <label for="igdbid">IGDB Client ID <span class="muted-xs">(Optional)</span></label>
              <input id="igdbid" bind:value={form.igdbClientId} placeholder="Twitch client ID" />
            </div>
            <div class="field" style="flex:1">
              <label for="igdbsec">IGDB Secret <span class="muted-xs">(Optional)</span></label>
              <input id="igdbsec" type="password" bind:value={form.igdbClientSecret} placeholder="Twitch client secret" />
            </div>
          </div>
        </section>
      </div>

      <!-- Right Column -->
      <div class="settings-col">
        <!-- Preferences & Offline Cache -->
        <section class="glass-strong card">
          <h2 class="section-title" style="margin-bottom:0.5rem">Local Storage &amp; Cache Architecture</h2>
          <p class="muted-sm" style="margin-bottom:1rem">
            Universal Game Launcher operates completely non-invasively. Native launcher files (Steam, Epic, GOG) are never modified. All collections, tags, custom art, and launcher preferences are safely persisted in your local fast-tier SQLite database and in-memory Redis KV cache.
          </p>
          <div class="kv">
            <div>
              <strong>In-Memory Redis KV Entries</strong>
              <p class="muted-xs" style="margin:0">Sub-millisecond fast-path cache keys.</p>
            </div>
            <span>{cacheStats?.totalKvEntries ?? 0} keys</span>
          </div>
          <div class="kv">
            <div>
              <strong>Cached Metadata Records</strong>
              <p class="muted-xs" style="margin:0">Locally stored SQLite metadata records.</p>
            </div>
            <span>{cacheStats?.totalMetadataCached ?? 0} records</span>
          </div>
          <div class="kv">
            <div>
              <strong>Cached Artwork Assets</strong>
              <p class="muted-xs" style="margin:0">Persistent offline grid &amp; hero artwork covers.</p>
            </div>
            <span>{cacheStats?.totalCachedArtworks ?? 0} files</span>
          </div>

          <div style="margin-top: 1.25rem; padding-top: 1rem; border-top: 1px solid rgba(255, 255, 255, 0.08);">
            <div class="row-between" style="align-items: center; margin-bottom: 0.35rem;">
              <strong style="font-size: 0.95rem;">{$t("settings.refreshMetadata")}</strong>
              {#if syncingLibraryMeta}
                <span class="pill pill-warn">{$t("settings.refreshMetadataRunning")}</span>
              {/if}
            </div>
            <p class="muted-xs" style="margin-bottom: 0.75rem;">
              {$t("settings.refreshMetadataHelp")}
            </p>
            <label class="row" style="gap: 0.5rem; margin-bottom: 0.75rem; cursor: pointer;">
              <input type="checkbox" bind:checked={syncLibraryAutoArtwork} style="width: auto;" />
              <span class="muted-sm">{$t("settings.alsoFetchArtwork")}</span>
            </label>
            <div class="row" style="gap: 0.75rem; align-items: center;">
              <button class="btn mini primary" onclick={refreshAllMetadata} disabled={syncingLibraryMeta}>
                {syncingLibraryMeta ? $t("settings.refreshMetadataRunning") : $t("settings.refreshMetadata")}
              </button>
              {#if syncLibraryMsg}
                <span class="muted-sm">{syncLibraryMsg}</span>
              {/if}
            </div>
          </div>

          <div style="margin-top: 1.25rem; padding-top: 1rem; border-top: 1px solid rgba(255, 255, 255, 0.08);">
            <div class="row-between" style="align-items: center; margin-bottom: 0.35rem;">
              <strong style="font-size: 0.95rem; color: #ff6b6b;">Clear &amp; Reset Metadata Cache</strong>
              {#if clearingCache}
                <span class="pill pill-warn">Purging &amp; Resetting…</span>
              {/if}
            </div>
            <p class="muted-xs" style="margin-bottom: 0.75rem;">
              Purges in-memory Redis KV entries, stored artwork, and cached metadata records if stale or corrupted. Clean metadata and artwork will be re-fetched as you browse.
            </p>
            {#if !showClearCacheConfirm}
              <div class="row" style="gap: 0.75rem; align-items: center;">
                <button class="btn mini" style="border-color: rgba(255,107,107,0.4); color: #ff8585;" onclick={() => (showClearCacheConfirm = true)} disabled={clearingCache}>
                  Purge &amp; Reset All Caches…
                </button>
                {#if clearCacheMsg}
                  <span class="muted-sm">{clearCacheMsg}</span>
                {/if}
              </div>
            {:else}
              <div style="padding: 0.75rem 1rem; background: rgba(255, 107, 107, 0.1); border: 1px solid rgba(255, 107, 107, 0.3); border-radius: 8px;">
                <p style="font-size: 0.85rem; margin: 0 0 0.75rem 0; color: #ffc9c9;">
                  Are you sure? This will delete all downloaded artwork covers, cached metadata, and in-memory KV records. Clean metadata and artwork will be re-fetched as you browse.
                </p>
                <div class="row" style="gap: 0.5rem; align-items: center;">
                  <button class="btn mini" style="background: #e03131; border-color: #c92a2a; color: white;" onclick={handleClearCache} disabled={clearingCache}>
                    {clearingCache ? "Purging…" : "Yes, Purge and Reset"}
                  </button>
                  <button class="btn mini" onclick={() => (showClearCacheConfirm = false)} disabled={clearingCache}>
                    Cancel
                  </button>
                </div>
              </div>
            {/if}
          </div>
        </section>
      </div>
    </div>
  {:else if activeTab === "platforms"}
    <PlatformsSection
      bind:sources
      bind:freeGameNotifications={form.enableFreeGameNotifications}
      {addRoot}
      {removeRoot}
      {setRoot}
    />
  {:else if activeTab === "plugins"}
    <div class="plugins-panel">
      <!-- Header row with Open Plugins Folder & Refresh buttons -->
      <div class="row-between" style="align-items: center; margin-bottom: 1.25rem;">
        <div>
          <h2 style="margin: 0; font-size: 1.2rem;">Drop-In Plugins &amp; Extensions</h2>
          <p class="muted-sm" style="margin: 0.25rem 0 0 0;">
            Universal Game Launcher loads plugins dynamically from drop-in manifests (<code>index.json</code>).
          </p>
        </div>
        <div class="row" style="gap: 0.5rem;">
          <button class="btn mini" onclick={handleOpenPluginsFolder} title="Open local plugins folder on disk">
            📁 Open Plugins Folder
          </button>
          <button class="btn mini btn-ghost" onclick={loadPlugins} disabled={loadingPlugins} title="Refresh plugins list">
            🔄 Refresh
          </button>
        </div>
      </div>

      {#if pluginActionMsg}
        <div class="pill pill-accent" style="display: block; margin-bottom: 1rem; text-align: center;">
          {pluginActionMsg}
        </div>
      {/if}

      <!-- Install Plugin from URL Card -->
      <section class="glass-strong card" style="margin-bottom: 1.5rem;">
        <h3 class="section-title" style="margin-top: 0;">Install Community Plugin via GitHub / URL</h3>
        <p class="muted-xs" style="margin-bottom: 0.75rem;">
          Enter a URL pointing to a GitHub repository's entry point or direct <code>index.json</code> manifest file.
        </p>
        <div class="row" style="gap: 0.5rem; align-items: center;">
          <input
            type="text"
            placeholder="https://github.com/author/my-launcher-plugin/raw/main/index.json"
            bind:value={pluginUrlInput}
            style="flex: 1;"
          />
          <button
            class="btn btn-primary"
            onclick={handleInstallFromUrl}
            disabled={installingPlugin || !pluginUrlInput.trim()}
          >
            {installingPlugin ? "Installing…" : "Install Plugin"}
          </button>
        </div>
      </section>

      <!-- Plugins Filter Bar -->
      <div class="row-between" style="align-items: center; margin-bottom: 1rem;">
        <div class="row" style="gap: 0.35rem;">
          {#each ["all", "platform", "metadata", "media", "utility"] as pType}
            <button
              class="btn mini"
              class:btn-primary={pluginFilterType === pType}
              class:btn-ghost={pluginFilterType !== pType}
              onclick={() => (pluginFilterType = pType)}
              style="text-transform: capitalize;"
            >
              {pType}
            </button>
          {/each}
        </div>
        <span class="muted-xs">{installedPlugins.length} total plugins installed</span>
      </div>

      <!-- Plugins Grid -->
      <div class="plugins-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px;">
        {#each installedPlugins.filter(p => pluginFilterType === "all" || p.pluginType === pluginFilterType) as plugin (plugin.id)}
          <div class="glass-strong card plugin-card" class:disabled={!plugin.enabled} style="display: flex; flex-direction: column; justify-content: space-between;">
            <div>
              <div class="row-between" style="align-items: flex-start; margin-bottom: 0.5rem;">
                <div>
                  <div class="row" style="align-items: center; gap: 0.5rem; flex-wrap: wrap;">
                    <strong class="plugin-name">{plugin.name}</strong>
                    <span class="pill pill-dim" style="font-size: 0.7rem;">v{plugin.version}</span>
                    {#if plugin.builtin}
                      <span class="pill pill-accent2" style="font-size: 0.65rem;">Built-in</span>
                    {:else}
                      <span class="pill pill-ok" style="font-size: 0.65rem;">Community</span>
                    {/if}
                  </div>
                  <span class="muted-xs" style="display: block; margin-top: 0.2rem;">By {plugin.author}</span>
                </div>
                <button
                  class="btn mini"
                  class:btn-primary={plugin.enabled}
                  class:btn-ghost={!plugin.enabled}
                  onclick={() => handleTogglePlugin(plugin)}
                  title={plugin.enabled ? "Disable plugin" : "Enable plugin"}
                >
                  {plugin.enabled ? "Enabled" : "Disabled"}
                </button>
              </div>

              <p class="muted-sm" style="margin: 0.5rem 0; font-size: 0.8rem; line-height: 1.4;">{plugin.description}</p>
            </div>

            <div class="row-between plugin-card-footer" style="margin-top: 0.75rem; padding-top: 0.5rem; border-top: 1px solid rgba(255, 255, 255, 0.06); align-items: center;">
              <span class="pill pill-dim" style="font-size: 0.68rem; text-transform: uppercase;">
                {plugin.pluginType}
              </span>
              {#if plugin.homepage}
                <a href={plugin.homepage} target="_blank" rel="noreferrer" class="muted-xs" style="color: var(--accent);">
                  Repository ↗
                </a>
              {:else}
                <span class="muted-xs">Self-Contained</span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{:else}
  <p class="muted">Loading settings…</p>
{/if}
