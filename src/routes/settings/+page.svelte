<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { STEAM_LANGUAGES, setLocale, syncWithSettingsLanguage, t } from "$lib/i18n";
  import { collections, error, games, install, settings } from "$lib/stores";
  import type { CatalogStats, EmulatorConfig, Settings } from "$lib/types";
  import CollectionsSection from "$lib/components/settings/CollectionsSection.svelte";
  import PlatformsSection from "$lib/components/settings/PlatformsSection.svelte";
  import OrganizeSection from "$lib/components/settings/OrganizeSection.svelte";
  import "$styles/settings.scss";

  type SettingsTab = "general" | "collections" | "platforms" | "organize";
  let activeTab = $state<SettingsTab>("general");

  let form = $state<Settings | null>(null);
  let catalogStats = $state<CatalogStats | null>(null);
  let syncingCatalog = $state(false);
  let catalogSyncMsg = $state<string | null>(null);
  let syncingLibraryMeta = $state(false);
  let syncLibraryAutoArtwork = $state(true);
  let syncLibraryMsg = $state<string | null>(null);
  let busy = $state(false);
  let saved = $state(false);

  let emulatorConfigs = $state<EmulatorConfig[]>([]);
  let scaffoldPath = $state("");
  let scaffolding = $state(false);
  let scaffoldMsg = $state<string | null>(null);

  const popularEmulators = [
    { name: "EmulationStation (ES-DE)", exe: "ES-DE.exe" },
    { name: "RetroArch", exe: "retroarch.exe" },
    { name: "Dolphin (GameCube/Wii)", exe: "Dolphin.exe" },
    { name: "PCSX2 (PlayStation 2)", exe: "pcsx2-qtx64.exe" },
    { name: "DuckStation (PlayStation)", exe: "duckstation-qt-x64-ReleaseLTCG.exe" },
    { name: "PPSSPP (PSP)", exe: "PPSSPPWindows64.exe" },
    { name: "Citra (3DS)", exe: "citra-qt.exe" },
    { name: "Cemu (Wii U)", exe: "Cemu.exe" },
    { name: "Ryujinx (Switch)", exe: "Ryujinx.exe" },
    { name: "mGBA (Game Boy Advance)", exe: "mGBA.exe" },
    { name: "Custom", exe: "" }
  ];

  function addEmulatorConfig() {
    emulatorConfigs = [
      ...emulatorConfigs,
      {
        id: crypto.randomUUID(),
        name: "EmulationStation (ES-DE)",
        executablePath: "",
        romDirectories: [""],
        defaultArgs: ""
      }
    ];
  }

  function removeEmulatorConfig(index: number) {
    emulatorConfigs = emulatorConfigs.filter((_, i) => i !== index);
  }

  function addEmulatorRomDir(emu: EmulatorConfig) {
    emu.romDirectories = [...emu.romDirectories, ""];
    emulatorConfigs = [...emulatorConfigs];
  }

  function removeEmulatorRomDir(emu: EmulatorConfig, dirIndex: number) {
    emu.romDirectories = emu.romDirectories.filter((_, i) => i !== dirIndex);
    emulatorConfigs = [...emulatorConfigs];
  }

  function setEmulatorRomDir(emu: EmulatorConfig, dirIndex: number, val: string) {
    emu.romDirectories = emu.romDirectories.map((r, i) => (i === dirIndex ? val : r));
    emulatorConfigs = [...emulatorConfigs];
  }

  async function scaffoldEsDe() {
    if (!scaffoldPath.trim()) return;
    scaffolding = true;
    scaffoldMsg = null;
    try {
      const dirs = await api.scaffoldEsDeDirectories(scaffoldPath.trim());
      scaffoldMsg = `Successfully scaffolded ${dirs.length} standard ES-DE system directories in ${scaffoldPath}!`;
    } catch (e) {
      scaffoldMsg = errorMessage(e);
    } finally {
      scaffolding = false;
    }
  }

  async function syncOnline() {
    syncingCatalog = true;
    catalogSyncMsg = null;
    try {
      const count = await api.syncOnlineCatalog();
      catalogStats = await api.getCatalogStats();
      catalogSyncMsg = `Synced ${count} games from official community catalog.`;
    } catch (e) {
      catalogSyncMsg = errorMessage(e);
    } finally {
      syncingCatalog = false;
    }
  }

  async function refreshAllMetadata() {
    syncingLibraryMeta = true;
    syncLibraryMsg = null;
    try {
      const res = await api.syncLibraryMetadata(form?.accountId3 ?? undefined, true, syncLibraryAutoArtwork);
      syncLibraryMsg = `Updated ${res.fetched} games (${res.cached} cached, ${res.unavailable} unresolved).`;
      catalogStats = await api.getCatalogStats().catch(() => null);
    } catch (e) {
      syncLibraryMsg = errorMessage(e);
    } finally {
      syncingLibraryMeta = false;
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
    { id: "emulator", label: "Emulators & ROMs" },
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
      if (tabParam && ["general", "collections", "platforms", "organize"].includes(tabParam)) {
        activeTab = tabParam;
      }

      const [loaded, detected, stats] = await Promise.all([
        api.getSettings(),
        api.detectSteam(),
        api.getCatalogStats().catch(() => null)
      ]);
      catalogStats = stats;
      settings.set(loaded);
      form = { ...loaded };
      if (loaded.language) {
        syncWithSettingsLanguage(loaded.language);
      }
      install.set(detected);
      if (!form.steamRoot && detected) form.steamRoot = detected.root;
      if (!form.accountId3 && detected?.accounts[0]) form.accountId3 = detected.accounts[0].id3;
      emulatorConfigs = (loaded.emulatorConfigs ?? []).map((e) => ({
        id: e.id || crypto.randomUUID(),
        name: e.name || "EmulationStation (ES-DE)",
        executablePath: e.executablePath || "",
        romDirectories: [...(e.romDirectories ?? [""])],
        defaultArgs: e.defaultArgs ?? ""
      }));
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
    form.emulatorConfigs = emulatorConfigs
      .filter((e) => e.name.trim() || e.executablePath.trim() || e.romDirectories.some((r) => r.trim()))
      .map((e) => ({
        id: e.id,
        name: e.name.trim(),
        executablePath: e.executablePath.trim(),
        romDirectories: e.romDirectories.map((r) => r.trim()).filter(Boolean),
        defaultArgs: e.defaultArgs?.trim() || null
      }));
    busy = true;
    try {
      const updated = await api.saveSettings(form);
      settings.set(updated);
      form = { ...updated };
      emulatorConfigs = (updated.emulatorConfigs ?? []).map((e) => ({
        id: e.id || crypto.randomUUID(),
        name: e.name || "EmulationStation (ES-DE)",
        executablePath: e.executablePath || "",
        romDirectories: [...(e.romDirectories ?? [""])],
        defaultArgs: e.defaultArgs ?? ""
      }));
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
    <span>{$t("settings.title")} &amp; Storage</span>
  </button>
  <button class="tab-btn" class:active={activeTab === "collections"} onclick={() => setTab("collections")}>
    <span class="tab-icon">❐</span>
    <span>{$t("nav.collections")}</span>
  </button>
  <button class="tab-btn" class:active={activeTab === "platforms"} onclick={() => setTab("platforms")}>
    <span class="tab-icon">⬡</span>
    <span>{$t("nav.platforms")}</span>
  </button>
  <button class="tab-btn" class:active={activeTab === "organize"} onclick={() => setTab("organize")}>
    <span class="tab-icon">⊞</span>
    <span>{$t("nav.organize")}</span>
  </button>
</div>

{#if form}
  {#if activeTab === "general"}
    <div class="settings-grid">
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
        <div class="row" style="gap:1rem">
          <div class="field" style="flex:1">
            <label for="cc">{$t("settings.storeCountry")}</label>
            <input id="cc" bind:value={form.countryCode} placeholder="us" />
          </div>
          <div class="field" style="flex:1">
            <label for="lang">{$t("settings.language")}</label>
            <select
              id="lang"
              bind:value={form.language}
              onchange={(e) => {
                if (form) {
                  form.language = e.currentTarget.value;
                  setLocale(e.currentTarget.value);
                }
              }}
            >
              {#each STEAM_LANGUAGES as lang (lang.id)}
                <option value={lang.id}>
                  {lang.nativeName} ({lang.name})
                </option>
              {/each}
            </select>
          </div>
        </div>
        {#if $install}
          <div class="kv" style="margin-top:0.5rem"><span class="muted">{$t("settings.librariesCount")}</span><span>{$install.libraries.length} folders</span></div>
          <div class="kv"><span class="muted">{$t("settings.accountsCount")}</span><span>{$install.accounts.length} accounts</span></div>
        {/if}
      </section>

      <!-- Storage & Drive Overview -->
      <section class="glass-strong card">
        <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
          <h2 class="section-title" style="margin:0">Storage &amp; Drive Overview</h2>
          <a href="/storage" class="btn mini primary" style="text-decoration:none">Open Storage Manager ↗</a>
        </div>
        <p class="muted-sm">
          Summary of detected Steam storage drives and registered library folders across your device.
        </p>
        <div class="kv">
          <span class="muted">Registered Steam Libraries</span>
          <span>{$install?.libraries.length ?? 0} folders</span>
        </div>
        <div class="kv">
          <span class="muted">Installed Games Tracked</span>
          <span>{$games.filter((g) => g.installed).length} games</span>
        </div>
        <div class="folder-list" style="margin-top:0.75rem">
          {#each $install?.libraries ?? [] as lib}
            <div class="folder-row" style="padding:0.4rem 0.6rem;background:rgba(255,255,255,0.03);border-radius:6px;justify-content:space-between">
              <span style="font-family:monospace;font-size:0.8rem;color:var(--text);overflow:hidden;text-overflow:ellipsis">{lib.path}</span>
              <span class="pill" style="font-size:0.65rem">{lib.appIds.length} apps</span>
            </div>
          {/each}
        </div>
      </section>

      <!-- API Keys -->
      <section class="glass-strong card">
        <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
          <h2 class="section-title" style="margin:0">API &amp; Storefront Keys</h2>
          <span class="pill" style="font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em">Keyless Architecture</span>
        </div>
        <p class="muted-sm" style="margin-bottom:0.75rem">
          Steam Vault uses local storefront integrations (Steam, GOG, Epic, Xbox, itch) and the offline Community Game Catalogue. Third-party API keys are strictly optional.
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

      <!-- Preferences & Offline Cache -->
      <section class="glass-strong card">
        <h2 class="section-title" style="margin-bottom:0.5rem">Preferences &amp; Offline Cache</h2>
        <label class="row" style="gap:0.5rem;margin-bottom:1rem;cursor:pointer">
          <input type="checkbox" bind:checked={form.autoBackup} style="width:auto" />
          <span>Back up Steam files before writing</span>
        </label>
        <p class="muted-sm" style="margin-bottom:1rem">
          Writes go to <code>cloud-storage-namespace-1.json</code> and <code>shortcuts.vdf</code>. Changes require Steam to be closed, and are reversible from the local backup folder.
        </p>
        <div class="kv">
          <div>
            <strong>Catalog Games Indexed</strong>
            <p class="muted-xs" style="margin:0">Instant offline game disambiguation.</p>
          </div>
          <span>{catalogStats?.totalGames ?? 0} games</span>
        </div>
        <div class="kv">
          <div>
            <strong>Cached Metadata Records</strong>
            <p class="muted-xs" style="margin:0">Locally stored metadata records.</p>
          </div>
          <span>{catalogStats?.totalMetadataCached ?? 0} records</span>
        </div>
        <div class="row" style="margin-top: 1rem; gap: 0.75rem; align-items: center;">
          <button class="btn mini" onclick={syncOnline} disabled={syncingCatalog}>
            {syncingCatalog ? $t("common.loading") : $t("settings.catalogSync")}
          </button>
          {#if catalogSyncMsg}
            <span class="muted-sm">{catalogSyncMsg}</span>
          {/if}
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
      </section>
    </div>
  {:else if activeTab === "collections"}
    <CollectionsSection />
  {:else if activeTab === "platforms"}
    <PlatformsSection
      bind:sources
      bind:emulatorConfigs
      {popularEmulators}
      bind:scaffoldPath
      bind:scaffolding
      bind:scaffoldMsg
      {scaffoldEsDe}
      {addEmulatorConfig}
      {removeEmulatorConfig}
      {addEmulatorRomDir}
      {removeEmulatorRomDir}
      {setEmulatorRomDir}
      {addRoot}
      {removeRoot}
      {setRoot}
    />
  {:else if activeTab === "organize"}
    <OrganizeSection />
  {/if}
{:else}
  <p class="muted">Loading settings…</p>
{/if}
