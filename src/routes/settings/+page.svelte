<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { STEAM_LANGUAGES, setLocale, syncWithSettingsLanguage, t } from "$lib/i18n";
  import { collections, error, games, install, settings } from "$lib/stores";
  import type { CatalogStats, EmulatorConfig, Settings } from "$lib/types";
  import "$styles/settings.scss";

  let form: Settings | null = null;
  let catalogStats: CatalogStats | null = null;
  let syncingCatalog = false;
  let catalogSyncMsg: string | null = null;
  let busy = false;
  let saved = false;

  let emulatorConfigs: EmulatorConfig[] = [];
  let scaffoldPath = "";
  let scaffolding = false;
  let scaffoldMsg: string | null = null;

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
      <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
        <h2 class="section-title" style="margin:0">API keys</h2>
        <span class="pill" style="font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em">Keyless Architecture</span>
      </div>
      <p class="muted-sm" style="margin-bottom:0.75rem">
        Steam Vault uses local storefront integrations (Steam, GOG, Epic, Xbox, itch) and the offline Community Game Catalogue. No third-party API keys are required for daily organizing or game launching.
      </p>
      <div class="field">
        <label for="steamkey">Steam Web API key <span class="muted-xs">(Optional — for remote whole-library sync)</span></label>
        <input id="steamkey" type="password" bind:value={form.steamApiKey} placeholder="Optional — steamcommunity.com/dev/apikey" />
      </div>
      <div class="field">
        <label for="sgdb">SteamGridDB key <span class="muted-xs">(Optional legacy fallback)</span></label>
        <input id="sgdb" type="password" bind:value={form.steamgriddbApiKey} placeholder="SteamGridDB API key" />
      </div>
      <div class="field">
        <label for="igdbid">IGDB (Twitch) client id <span class="muted-xs">(Optional legacy fallback)</span></label>
        <input id="igdbid" bind:value={form.igdbClientId} placeholder="Twitch client id" />
      </div>
      <div class="field">
        <label for="igdbsec">IGDB (Twitch) client secret <span class="muted-xs">(Optional legacy fallback)</span></label>
        <input id="igdbsec" type="password" bind:value={form.igdbClientSecret} placeholder="Twitch client secret" />
      </div>
    </section>

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
          <div class="folder-row" style="padding:0.4rem 0.6rem;background:rgba(255,255,255,0.03);border-radius:6px">
            <span style="font-family:monospace;font-size:0.8rem;color:var(--text)">{lib.path}</span>
            <span class="pill" style="font-size:0.65rem">{lib.appIds.length} apps</span>
          </div>
        {/each}
      </div>
    </section>

    <section class="glass-strong card">
      <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
        <h2 class="section-title" style="margin:0">Organization &amp; Scopes</h2>
        <a href="/organize" class="btn mini primary" style="text-decoration:none">Open Library Organizer ↗</a>
      </div>
      <p class="muted-sm">
        Multi-scope auto-categorization and Depressurizer-style custom rule builder.
      </p>
      <div class="kv">
        <span class="muted">Active Steam Collections</span>
        <span>{$collections.length} collections</span>
      </div>
      <div class="kv">
        <span class="muted">Total Games in Library</span>
        <span>{$games.length} games</span>
      </div>
      <div class="kv">
        <span class="muted">Supported Scopes</span>
        <span>Genre, Developer, Publisher, Year, Platform, Custom Rules</span>
      </div>
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">Storefront & Platform Integrations</h2>
      <p class="muted-sm">
        Automatically detect games from installed launchers and storefront integrations. Toggle each platform on or off. All default and user installation locations are automatically discovered through each platform's own configuration.
      </p>
      <div class="integrations-grid">
        {#each sources.filter((s) => s.id !== "other" && s.id !== "emulator") as source (source.id)}
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

    {#if sources.find((s) => s.id === "emulator")}
      {#each [sources.find((s) => s.id === "emulator")!] as emuSource (emuSource.id)}
      <section class="glass-strong card full-width">
        <div class="source-head" style="margin-bottom:0.5rem">
          <div>
            <div class="row" style="gap:0.6rem;align-items:center">
              <h2 class="section-title" style="margin:0">EmulationStation (ES-DE) &amp; ROM Directories</h2>
              <span class="pill" style="font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em">Manual Configuration Policy</span>
            </div>
            <p class="muted-sm" style="margin-top:0.35rem">
              Steam Vault uses <strong>EmulationStation Desktop Edition (ES-DE)</strong> as the primary supported method for organizing, launching, and managing ROMs.
              Automatic disk sweeps are disabled to respect user privacy and prevent unauthorized scans. Explicitly add your emulator executables and ROM folders below.
            </p>
          </div>
          <label class="row" style="gap:0.5rem;margin:0;cursor:pointer">
            <input
              type="checkbox"
              checked={emuSource.enabled}
              style="width:auto"
              onchange={(event) => {
                emuSource.enabled = event.currentTarget.checked;
                sources = [...sources];
              }}
            />
            <strong>Enabled</strong>
          </label>
        </div>

        <!-- ES-DE Directory Scaffolding Box -->
        <div class="rom-dirs-box" style="margin-top:0.75rem">
          <div class="row-between" style="align-items:center;margin-bottom:0.35rem">
            <div>
              <strong style="font-size:0.88rem">Scaffold Standard ES-DE ROMs Directory Structure</strong>
              <p class="muted-xs" style="margin:0.15rem 0 0">
                Initializes the official <a href="https://github.com/retrogamecorps/ES-DE-Directories" target="_blank" rel="noreferrer" style="color:var(--accent)">ES-DE directory layout</a> (50+ console system folders with systeminfo.txt guides) inside your chosen folder.
              </p>
            </div>
            <span class="pill" style="font-size:0.68rem">ES-DE Standard</span>
          </div>
          <div class="row" style="gap:0.5rem;margin-top:0.5rem">
            <input
              style="flex:1"
              placeholder="e.g. D:\Emulation or E:\ROMs"
              bind:value={scaffoldPath}
            />
            <button
              class="btn mini primary"
              disabled={scaffolding || !scaffoldPath.trim()}
              onclick={scaffoldEsDe}
            >
              {scaffolding ? "Scaffolding…" : "Scaffold Directory"}
            </button>
          </div>
          {#if scaffoldMsg}
            <p class="muted-xs" style="margin-top:0.4rem;color:var(--accent)">{scaffoldMsg}</p>
          {/if}
        </div>

        <!-- Configured Emulators List -->
        <div style="margin-top:1.1rem">
          <div class="row-between" style="align-items:center;margin-bottom:0.5rem">
            <h3 style="font-size:0.95rem;margin:0;font-weight:600">Configured Emulators &amp; Executables</h3>
            <button class="btn mini" onclick={addEmulatorConfig}>＋ Add Emulator</button>
          </div>

          {#if emulatorConfigs.length === 0}
            <p class="muted-sm" style="margin:0.35rem 0">
              No emulators configured yet. Click "Add Emulator" to register EmulationStation (ES-DE) or standalone emulators (Dolphin, PCSX2, RetroArch, etc.).
            </p>
          {/if}

          {#each emulatorConfigs as emu, index (emu.id)}
            <div class="emulator-card">
              <div class="emulator-header">
                <div class="emulator-title">
                  <select
                    style="font-weight:600;padding:0.3rem 0.5rem;border-radius:4px"
                    bind:value={emu.name}
                  >
                    {#each popularEmulators as pop}
                      <option value={pop.name}>{pop.name}</option>
                    {/each}
                  </select>
                </div>
                <button
                  class="btn mini danger-ghost"
                  title="Remove this emulator"
                  onclick={() => removeEmulatorConfig(index)}
                >
                  ✕ Remove
                </button>
              </div>

              <div class="emulator-grid">
                <div class="field" style="margin:0">
                  <label for={"emu-exe-" + emu.id}>Executable Path</label>
                  <input
                    id={"emu-exe-" + emu.id}
                    placeholder="e.g. C:\Emulation\ES-DE\ES-DE.exe"
                    bind:value={emu.executablePath}
                  />
                </div>
                <div class="field" style="margin:0">
                  <label for={"emu-args-" + emu.id}>Launch Options / Arguments (Optional)</label>
                  <input
                    id={"emu-args-" + emu.id}
                    placeholder="Default launch options (leave empty for standard)"
                    bind:value={emu.defaultArgs}
                  />
                </div>
              </div>

              <div>
                <div class="row-between" style="align-items:center;margin-bottom:0.3rem">
                  <span class="muted-xs" style="font-weight:600">ROM Folders Scanned for this Emulator</span>
                  <button class="btn mini" style="font-size:0.75rem;padding:0.2rem 0.45rem" onclick={() => addEmulatorRomDir(emu)}>＋ Add ROM folder</button>
                </div>
                <div class="folder-list" style="margin-top:0.2rem">
                  {#each emu.romDirectories as romDir, dirIdx}
                    <div class="folder-row">
                      <input
                        placeholder="e.g. D:\ROMs\gc or D:\Emulation\ROMs"
                        value={romDir}
                        oninput={(e) => setEmulatorRomDir(emu, dirIdx, e.currentTarget.value)}
                      />
                      <button
                        class="btn mini"
                        title="Remove ROM folder"
                        onclick={() => removeEmulatorRomDir(emu, dirIdx)}
                      >
                        ×
                      </button>
                    </div>
                  {/each}
                  {#if emu.romDirectories.length === 0}
                    <p class="muted-xs" style="margin:0">No ROM folders added for this emulator.</p>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
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
