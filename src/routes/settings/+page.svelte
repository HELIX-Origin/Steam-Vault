<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { STEAM_LANGUAGES, setLocale, syncWithSettingsLanguage, t } from "$lib/i18n";
  import { error, install, settings } from "$lib/stores";
  import type { CatalogStats, Settings } from "$lib/types";

  let form: Settings | null = null;
  let catalogStats: CatalogStats | null = null;
  let syncingCatalog = false;
  let catalogSyncMsg: string | null = null;
  let busy = false;
  let saved = false;

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
  let sources: SourceRow[] = [];
  let finding: string | null = null;

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

  /** Autofill a platform's detected library folders, like the Steam Auto-detect. */
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

  onMount(async () => {
    try {
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

<div class="row-between" style="margin-bottom:1.1rem">
  <div>
    <h1>{$t("settings.title")}</h1>
    <p class="muted">{$t("settings.subtitle")}</p>
  </div>
  <div class="row">
    {#if saved}<span class="pill">{$t("common.saved")}</span>{/if}
    <button class="btn btn-primary" disabled={busy || !form} onclick={save}>{busy ? $t("common.saving") : $t("common.save")}</button>
  </div>
</div>

{#if form}
  <div class="grid two">
    <section class="glass-strong card">
      <h2 class="section-title">{$t("settings.steamFolder")}</h2>
      <div class="field">
        <label for="root">{$t("settings.steamFolder")}</label>
        <input id="root" bind:value={form.steamRoot} placeholder="C:\Program Files (x86)\Steam" />
      </div>
      <button class="btn" onclick={detect}>{$t("settings.autoDetect")}</button>
      {#if $install}
        <div class="kv"><span class="muted">{$t("settings.librariesCount")}</span><span>{$install.libraries.length}</span></div>
        <div class="kv"><span class="muted">{$t("settings.accountsCount")}</span><span>{$install.accounts.length}</span></div>
      {/if}
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">{$t("settings.activeAccount")}</h2>
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
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">API keys</h2>
      <div class="field">
        <label for="steamkey">Steam Web API key</label>
        <input id="steamkey" type="password" bind:value={form.steamApiKey} placeholder="Get one at steamcommunity.com/dev/apikey" />
      </div>
      <div class="field">
        <label for="sgdb">SteamGridDB key</label>
        <input id="sgdb" type="password" bind:value={form.steamgriddbApiKey} placeholder="SteamGridDB API key" />
      </div>
      <div class="field">
        <label for="igdbid">IGDB (Twitch) client id</label>
        <input id="igdbid" bind:value={form.igdbClientId} placeholder="Twitch client id" />
      </div>
      <div class="field">
        <label for="igdbsec">IGDB (Twitch) client secret</label>
        <input id="igdbsec" type="password" bind:value={form.igdbClientSecret} placeholder="Twitch client secret" />
      </div>
      <p class="muted-sm">Steam Web API, SteamGridDB, and IGDB are the only providers that need keys.</p>
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">Storefront & Platform Integrations</h2>
      <p class="muted-sm">
        Automatically detect games from installed launchers and storefront integrations. Toggle each platform on or off. All default and user installation locations are automatically discovered through each platform's own configuration.
      </p>
      <div class="integrations-grid">
        {#each sources.filter((s) => s.id !== "other") as source (source.id)}
          <label class="integration-item">
            <input
              type="checkbox"
              checked={source.enabled}
              onchange={(event) => {
                source.enabled = event.currentTarget.checked;
                sources = [...sources];
              }}
            />
            <div class="integration-label">
              <strong>{source.label}</strong>
              <span class="muted-xs">{source.enabled ? "Active" : "Disabled"}</span>
            </div>
          </label>
        {/each}
      </div>
    </section>

    {#if sources.find((s) => s.id === "other")}
      {#each [sources.find((s) => s.id === "other")!] as other (other.id)}
      <section class="glass-strong card">
        <div class="source-head" style="margin-bottom:0.5rem">
          <div>
            <h2 class="section-title">Local & Standalone Games</h2>
            <p class="muted-sm">
              Manual path entries for local games that have no custom installer or launcher. Steam Vault scans these folders for games and executables using the folder name as the game title.
            </p>
          </div>
          <label class="row" style="gap:0.5rem;margin:0;cursor:pointer">
            <input
              type="checkbox"
              checked={other.enabled}
              style="width:auto"
              onchange={(event) => {
                other.enabled = event.currentTarget.checked;
                sources = [...sources];
              }}
            />
            <strong>Enabled</strong>
          </label>
        </div>
        {#if other.roots.length === 0}
          <p class="muted-sm" style="margin:0.25rem 0">No local game folders configured.</p>
        {/if}
        <div class="folder-list">
          {#each other.roots as folder, index}
            <div class="folder-row">
              <input
                placeholder="C:\Games\Standalone"
                value={folder}
                oninput={(event) => setRoot(other, index, event.currentTarget.value)}
              />
              <button class="btn mini" title="Remove folder" onclick={() => removeRoot(other, index)}>×</button>
            </div>
          {/each}
          <button class="btn mini add" onclick={() => addRoot(other)}>＋ Add folder</button>
        </div>
      </section>
      {/each}
    {/if}

    <section class="glass-strong card">
      <h2 class="section-title">Preferences</h2>
      <label class="row" style="gap:0.5rem;margin-bottom:1rem">
        <input type="checkbox" bind:checked={form.autoBackup} style="width:auto" />
        Back up Steam files before writing
      </label>
      <p class="muted-sm">
        Writes go to <code>cloud-storage-namespace-1.json</code> and <code>shortcuts.vdf</code>. Changes require
        Steam to be closed, and are reversible from the local backup folder.
      </p>
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">Local Game Catalog & Cache</h2>
      <div class="kv">
        <div>
          <strong>Catalog Games</strong>
          <p class="muted-sm">Indexed titles and executables for instant offline disambiguation.</p>
        </div>
        <span>{catalogStats?.totalGames ?? 0} games</span>
      </div>
      <div class="kv">
        <div>
          <strong>Cached Metadata</strong>
          <p class="muted-sm">Locally stored store and community metadata records.</p>
        </div>
        <span>{catalogStats?.totalMetadataCached ?? 0} records</span>
      </div>
      <div class="row" style="margin-top: 1rem; gap: 0.75rem; align-items: center;">
        <button class="btn mini primary" onclick={syncOnline} disabled={syncingCatalog}>
          {syncingCatalog ? "Syncing…" : "Sync Online Catalog"}
        </button>
        {#if catalogSyncMsg}
          <span class="muted-sm">{catalogSyncMsg}</span>
        {/if}
      </div>
    </section>
  </div>
{:else}
  <p class="muted">Loading settings…</p>
{/if}

<style>
  .two {
    grid-template-columns: 1fr 1fr;
    align-items: start;
  }

  .card {
    padding: 1.2rem 1.35rem;
  }

  .kv {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border-subtle);
    margin-top: 0.4rem;
  }

  .kv:last-child {
    border-bottom: none;
  }

  .integrations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 0.65rem;
    margin-top: 0.85rem;
  }

  .integration-item {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.65rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: rgba(6, 14, 26, 0.45);
    cursor: pointer;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .integration-item:hover {
    border-color: var(--border-strong);
    background: rgba(6, 14, 26, 0.65);
  }

  .integration-item input[type="checkbox"] {
    margin: 0;
    width: auto;
    cursor: pointer;
  }

  .integration-label {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .muted-xs {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .folder-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin-top: 0.6rem;
  }

  .source-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .folder-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .folder-row input {
    flex: 1;
  }

  .btn.mini {
    padding: 0.32rem 0.65rem;
    font-size: 0.82rem;
    line-height: 1.2;
  }

  .btn.add {
    align-self: flex-start;
  }
</style>
