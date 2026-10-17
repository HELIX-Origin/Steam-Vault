<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { listen } from "@tauri-apps/api/event";
  import { error, platformScan } from "$lib/stores";
  import { PLATFORM_PLUGINS } from "$lib/plugins";
  import PlatformIcon from "$lib/components/PlatformIcon.svelte";
  import type { EosOverlayStatus, LauncherStatus, LegendaryAuthStatus, ManagedDirectories, Platform, PlatformGame } from "$lib/types";

  type SourceRow = { id: string; label: string; enabled: boolean; roots: string[] };

  let {
    sources = $bindable(),
    freeGameNotifications = $bindable(false),
    addRoot,
    removeRoot,
    setRoot
  }: {
    sources: SourceRow[];
    freeGameNotifications?: boolean;
    addRoot: (source: SourceRow) => void;
    removeRoot: (source: SourceRow, index: number) => void;
    setRoot: (source: SourceRow, index: number, value: string) => void;
  } = $props();

  const labels: Record<string, string> = {
    steam: "Steam",
    gog: "GOG",
    epic: "Epic Games",
    itch: "itch.io",
    ubisoft: "Ubisoft Connect",
    ea: "EA",
    battlenet: "Battle.net",
    amazon: "Amazon Games",
    xbox: "Xbox",
    rockstar: "Rockstar",
    other: "Other"
  };

  let selected = $state(new Set<string>());
  let filter = $state<Platform | "all">("all");
  let busy = $state(false);
  let imported = $state<{ added: number; skipped: string[] } | null>(null);
  let includeOtherDrives = $state(false);

  let launcherStatuses = $state<Record<string, LauncherStatus>>({});
  let loadingLaunchers = $state(true);
  let installingLauncherId = $state<string | null>(null);
  let legendaryMsg = $state<string | null>(null);

  let legendaryAuth = $state<LegendaryAuthStatus | null>(null);
  let legendaryPathInput = $state("");
  let savingPath = $state(false);
  let loggingIn = $state(false);
  let authError = $state<string | null>(null);

  let showAdvancedLegendary = $state(false);
  let eosOverlayStatus = $state<EosOverlayStatus | null>(null);
  let installingEos = $state(false);

  let managedDirs = $state<ManagedDirectories | null>(null);
  let customToolsInput = $state("");
  let customLaunchersInput = $state("");
  let savingDirs = $state(false);

  async function loadLaunchers() {
    loadingLaunchers = true;
    try {
      const dirs = await api.getManagedDirectories().catch(() => null);
      if (dirs) {
        managedDirs = dirs;
        customToolsInput = dirs.toolsDir;
        customLaunchersInput = dirs.launchersDir;
      }

      const map: Record<string, LauncherStatus> = {};
      await Promise.all(
        PLATFORM_PLUGINS.map(async (plugin) => {
          const key = plugin.id.replace("platform-", "");
          try {
            const detected = await plugin.detectLauncher?.();
            map[key] = {
              id: key,
              name: plugin.name.replace(" Integration", "").replace(" Platform", ""),
              installed: detected?.installed ?? false,
              executablePath: detected?.executablePath,
              alternativeMethod: detected?.alternativeMethod,
              downloadUrl: plugin.downloadUrl ?? "",
              alternativeUrl: plugin.alternativeUrl,
              alternativeLabel: plugin.alternativeLabel,
              supportsSilentInstall: (plugin.supportsSilentInstall ?? false) && (dirs?.wingetAvailable ?? false),
              wingetPackageId: plugin.wingetPackageId,
            };
          } catch {
            map[key] = {
              id: key,
              name: plugin.name,
              installed: false,
              downloadUrl: plugin.downloadUrl ?? "",
              alternativeUrl: plugin.alternativeUrl,
              alternativeLabel: plugin.alternativeLabel,
              supportsSilentInstall: (plugin.supportsSilentInstall ?? false) && (dirs?.wingetAvailable ?? false),
              wingetPackageId: plugin.wingetPackageId,
            };
          }
        })
      );
      launcherStatuses = map;
    } catch (e) {
      console.warn("Could not inspect platform plugins:", e);
    } finally {
      loadingLaunchers = false;
    }
  }

  async function handleInstallLauncher(platformKey: string) {
    const plugin = PLATFORM_PLUGINS.find(
      (p) => p.id === `platform-${platformKey}` || p.id === platformKey
    );
    if (!plugin?.installLauncher) return;

    installingLauncherId = platformKey;
    legendaryMsg = `Installing ${plugin.name}…`;
    try {
      const targetLocation = managedDirs?.launchersDir || undefined;
      const res = await plugin.installLauncher({ targetDir: targetLocation });
      if (res.success) {
        legendaryMsg = res.message;
        await loadLaunchers();
      } else {
        error.set(res.message);
      }
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      installingLauncherId = null;
    }
  }

  async function handleSaveManagedDirs() {
    savingDirs = true;
    try {
      await api.setManagedDirectories(customToolsInput || null, customLaunchersInput || null);
      managedDirs = await api.getManagedDirectories();
      legendaryMsg = "Updated managed installation directories successfully.";
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      savingDirs = false;
    }
  }

  onMount(() => {
    loadLaunchers();
  });

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

  async function handleOpenLaunchersFolder() {
    try {
      await api.openLaunchersFolder();
    } catch (e) {
      error.set(errorMessage(e));
    }
  }

  function key(game: PlatformGame): string {
    return `${game.platform}|${game.installDir ?? game.exe ?? game.name}`.toLowerCase();
  }

  function toggle(game: PlatformGame) {
    const next = new Set(selected);
    const id = key(game);
    next.has(id) ? next.delete(id) : next.add(id);
    selected = next;
  }

  let scan = $derived($platformScan);
  let available = $derived(scan?.games ?? []);
  let shown = $derived(filter === "all" ? available : available.filter((game) => game.platform === filter));

  function selectVisible() {
    const next = new Set(selected);
    for (const game of shown) next.add(key(game));
    selected = next;
  }

  function clearSelection() {
    selected = new Set();
  }

  async function runScan() {
    busy = true;
    imported = null;
    try {
      platformScan.set(await api.detectPlatforms(includeOtherDrives));
      selected = new Set();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }

  async function importSelected() {
    const chosen = available.filter((game) => selected.has(key(game)));
    if (chosen.length === 0) return;
    busy = true;
    try {
      const result = await api.importPlatformGames(chosen);
      imported = { added: result.added.length, skipped: result.skipped };
      selected = new Set();
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="settings-platforms-section">
  <!-- Storefront & Platform Integrations -->
  <section class="glass-strong card" style="margin-bottom:1.25rem">
    <div class="row-between" style="align-items:center;margin-bottom:0.4rem">
      <h2 class="section-title" style="margin:0">Storefront &amp; Platform Integrations</h2>
      <div class="row" style="align-items:center;gap:0.5rem">
        <button class="btn mini btn-ghost" onclick={loadLaunchers} disabled={loadingLaunchers} title="Re-check installed launchers">
          {loadingLaunchers ? "Checking launchers…" : "↻ Check Launchers"}
        </button>
        <button
          class="btn mini btn-ghost"
          class:btn-accent={showAdvancedLegendary}
          onclick={() => (showAdvancedLegendary = !showAdvancedLegendary)}
          title="Configure Legendary CLI path and Epic Online Services overlay"
        >
          ⚙ Legendary &amp; EOS Settings
        </button>
        <button class="btn mini btn-ghost" onclick={handleOpenToolsFolder} title="Open user tools directory where helper binaries and CLI tools are stored">
          Tools Folder ↗
        </button>
        <button class="btn mini btn-ghost" onclick={handleOpenLaunchersFolder} title="Open user launchers directory where games and platform clients are installed">
          Launchers Folder ↗
        </button>
      </div>
    </div>
    <p class="muted-sm">
      Automatically detect games from installed launchers and native gaming services. Launchers are detected directly via binaries on PATH, system packages, or user-managed custom directories.
    </p>
    <div class="integrations-grid">
      {#each sources.filter((s) => s.id !== "other" && s.id !== "emulator") as source (source.id)}
        {@const launcher = launcherStatuses[source.id]}
        {@const isInstalled = launcher ? launcher.installed : true}
        <div class="integration-item glass" class:is-disabled={!isInstalled}>
          <label class="integration-toggle-row" class:not-allowed={!isInstalled}>
            <input
              type="checkbox"
              checked={source.enabled && isInstalled}
              disabled={!isInstalled}
              onchange={(event) => {
                if (!isInstalled) return;
                source.enabled = event.currentTarget.checked;
                sources = [...sources];
              }}
            />
            <div class="integration-label">
              <div class="row" style="align-items:center;gap:0.4rem;flex-wrap:wrap">
                <PlatformIcon platform={source.id} size={14} />
                <strong>{source.label}</strong>
                {#if launcher}
                  {#if isInstalled}
                    <span class="pill pill-ok" style="font-size:0.62rem">Installed</span>
                    {#if launcher.alternativeMethod}
                      <span class="pill pill-accent" style="font-size:0.62rem">{launcher.alternativeMethod}</span>
                    {/if}
                  {:else}
                    <span class="pill pill-dim" style="font-size:0.62rem">Not Installed</span>
                  {/if}
                {/if}
              </div>
              <span class="muted-xs">
                {#if !isInstalled}
                  Disabled — Launcher binary not detected on PATH or custom folders
                {:else if source.enabled}
                  Active &amp; scanned
                {:else}
                  Disabled
                {/if}
              </span>
            </div>
          </label>

          {#if !isInstalled && launcher}
            <div class="integration-actions-row" style="display:flex;gap:0.4rem;align-items:center;flex-wrap:wrap">
              {#if launcher.supportsSilentInstall}
                <button
                  class="btn mini btn-accent launcher-action-btn"
                  onclick={() => handleInstallLauncher(source.id)}
                  disabled={installingLauncherId === source.id}
                  title="Silently install {launcher.name} via winget into user-managed folder"
                >
                  {installingLauncherId === source.id ? "Installing…" : "⚡ Install"}
                </button>
              {/if}
              {#if launcher.downloadUrl}
                <button
                  class="btn mini btn-ghost launcher-action-btn"
                  onclick={() => handleOpenUrl(launcher.downloadUrl)}
                  title="Download official {launcher.name} via web browser"
                >
                  Download ↗
                </button>
              {/if}
              {#if launcher.alternativeUrl}
                <button
                  class="btn mini btn-ghost launcher-action-btn"
                  onclick={() => handleOpenUrl(launcher.alternativeUrl!)}
                  title={launcher.alternativeLabel ?? "Download CLI"}
                >
                  {launcher.alternativeLabel ?? "CLI ↗"}
                </button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- User-Managed Installation Directories -->
    <div class="glass card" style="margin-top:1rem;padding:0.85rem 1rem">
      <div class="row-between" style="align-items:center;margin-bottom:0.5rem">
        <div>
          <strong style="font-size:0.92rem">User-Manageable Folders &amp; Organization</strong>
          <p class="muted-xs" style="margin:0.2rem 0 0">
            Specify custom installation directories for tools, helpers, and downloaded launchers to maintain clean library organization.
          </p>
        </div>
        <button class="btn mini btn-primary" onclick={handleSaveManagedDirs} disabled={savingDirs}>
          {savingDirs ? "Saving…" : "Save Folders"}
        </button>
      </div>
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;margin-top:0.5rem">
        <div>
          <label for="custom-tools-folder" class="muted-xs" style="display:block;margin-bottom:0.25rem">Custom Tools &amp; Helper Binaries Folder</label>
          <div class="row" style="gap:0.35rem">
            <input
              id="custom-tools-folder"
              type="text"
              bind:value={customToolsInput}
              placeholder={managedDirs?.toolsDir ?? "Default tools directory"}
              style="flex:1;font-size:0.8rem;padding:0.35rem 0.5rem"
            />
            <button class="btn mini btn-ghost" onclick={handleOpenToolsFolder} title="Open folder">↗</button>
          </div>
        </div>
        <div>
          <label for="custom-launchers-folder" class="muted-xs" style="display:block;margin-bottom:0.25rem">Custom Platform Launchers Folder</label>
          <div class="row" style="gap:0.35rem">
            <input
              id="custom-launchers-folder"
              type="text"
              bind:value={customLaunchersInput}
              placeholder={managedDirs?.launchersDir ?? "Default launchers directory"}
              style="flex:1;font-size:0.8rem;padding:0.35rem 0.5rem"
            />
            <button class="btn mini btn-ghost" onclick={handleOpenLaunchersFolder} title="Open folder">↗</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Free Game Notifications & Promotions Opt-in Toggle -->
    <div class="glass card" style="margin-top:1rem;padding:0.85rem 1rem">
      <div class="row-between" style="align-items:center">
        <div>
          <strong style="font-size:0.92rem">Free Game Promotions &amp; Alerts</strong>
          <p class="muted-xs" style="margin:0.2rem 0 0">
            Show alerts on the Executive Dashboard when 100% free games are available to claim from your enabled storefronts (e.g. Epic Games weekly free titles, GOG giveaways).
          </p>
        </div>
        <label class="row" style="align-items:center;gap:0.5rem;cursor:pointer;user-select:none;margin:0">
          <input
            type="checkbox"
            bind:checked={freeGameNotifications}
            style="width:auto;margin:0"
          />
          <span class="muted-sm">{freeGameNotifications ? "Enabled" : "Disabled"}</span>
        </label>
      </div>
    </div>
    {#if legendaryMsg}
      <div style="margin-top:0.75rem;padding:0.5rem 0.75rem;background:rgba(56,189,248,0.1);border-radius:6px;font-size:0.8rem;color:var(--text)">
        {legendaryMsg}
      </div>
    {/if}
  </section>

  <!-- Non-Steam Platform Game Scanner & Importer -->
  <section class="glass-strong card" style="margin-bottom:1.25rem">
    <div class="row-between" style="align-items:center;margin-bottom:0.75rem">
      <div>
        <h2 class="section-title" style="margin:0">{$t("platforms.title")} &amp; Importer</h2>
        <p class="muted-sm" style="margin:0.25rem 0 0">
          Scan for installed games across GOG, Epic, itch.io, Amazon, EA, Battle.net, and ROM directories to import as Steam shortcuts.
        </p>
      </div>
      <div class="row" style="align-items:center;gap:0.75rem">
        <label style="display:inline-flex;align-items:center;gap:0.4rem;font-size:0.85rem;color:var(--text-muted);cursor:pointer;user-select:none">
          <input type="checkbox" bind:checked={includeOtherDrives} />
          Scan all drives
        </label>
        <button class="btn btn-primary" onclick={runScan} disabled={busy}>
          {busy ? $t("common.scanning") : $t("platforms.scanAll")}
        </button>
      </div>
    </div>

    {#if imported}
      <div class="glass-strong card" style="padding:0.75rem 1rem;margin-bottom:1rem;background:rgba(56,189,248,0.1);border-color:var(--border-glow)">
        <strong>{imported.added}</strong> game{imported.added === 1 ? "" : "s"} imported into Steam shortcuts.
        {#if imported.skipped.length}
          <span class="muted-sm"> {imported.skipped.length} skipped (already present or no executable).</span>
        {/if}
      </div>
    {/if}

    {#if scan}
      <div class="platform-cards" style="margin-bottom:1rem">
        {#each scan.platforms as status (status.platform)}
          <button
            class="glass platform-card"
            class:active={filter === status.platform}
            onclick={() => (filter = filter === status.platform ? "all" : status.platform)}
          >
            <div class="row-between">
              <span class="name" style="display:inline-flex;align-items:center;gap:0.35rem">
                <PlatformIcon platform={status.platform} size={14} />
                {status.label}
              </span>
              <span class="pill" class:pill-dim={status.count === 0}>{status.count}</span>
            </div>
            <span class="muted-sm">
              {#if status.available}
                {status.count} found
              {:else}
                no local data source
              {/if}
            </span>
            {#if status.notes}
              <span class="muted-sm note">{status.notes}</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="glass-strong toolbar" style="margin-bottom:1rem">
        <span class="muted-sm">
          {shown.length} game{shown.length === 1 ? "" : "s"} shown
          {#if filter !== "all"}· {labels[filter]}{/if}
        </span>
        <div class="spacer"></div>
        {#if filter !== "all"}
          <button class="btn btn-ghost mini" onclick={() => (filter = "all")}>Show all</button>
        {/if}
        <button class="btn mini" onclick={selectVisible} disabled={shown.length === 0}>Select shown</button>
        <button class="btn btn-ghost mini" onclick={clearSelection} disabled={selected.size === 0}>Clear</button>
        <button class="btn btn-primary mini" onclick={importSelected} disabled={busy || selected.size === 0}>
          Import {selected.size || ""} selected to Library
        </button>
      </div>

      <div class="glass platform-list">
        {#each shown as game (key(game))}
          <button class="row platform-game" class:selected={selected.has(key(game))} onclick={() => toggle(game)}>
            <span class="platform-check">{selected.has(key(game)) ? "✓" : ""}</span>
            <span class="platform-name" style="display:inline-flex;align-items:center;gap:0.4rem;flex-wrap:wrap">
              {game.name}
              <span class="pill pill-accent2" style="display:inline-flex;align-items:center;gap:0.3rem">
                <PlatformIcon platform={game.platform} consoleName={game.console} size={12} />
                {labels[game.platform]}
              </span>
              {#if game.console}
                <span class="pill pill-dim">{game.console}</span>
              {/if}
            </span>
            <span class="muted-sm src">{game.source ?? ""}</span>
          </button>
        {/each}
        {#if shown.length === 0}
          <p class="muted" style="padding:1rem">
            Nothing to import here. Try another platform, or run a fresh scan.
          </p>
        {/if}
      </div>
    {:else}
      <div class="glass" style="padding:1.5rem;text-align:center;border-radius:var(--radius-sm)">
        <p class="muted" style="margin:0 0 0.35rem">
          Click "Scan Non-Steam Games" to search for games installed across your launchers and system drives.
        </p>
        <p class="muted-sm" style="margin:0">
          Universal Game Launcher reads launcher manifests and local registries without launching background processes.
        </p>
      </div>
    {/if}
  </section>

  <!-- Local & Standalone Games -->
  {#if sources.find((s) => s.id === "other")}
    {#each [sources.find((s) => s.id === "other")!] as other (other.id)}
      <section class="glass-strong card" style="margin-bottom:1.25rem">
        <div class="source-head" style="margin-bottom:0.5rem">
          <div>
            <h2 class="section-title">Local &amp; Standalone Games</h2>
            <p class="muted-sm">
              Manual path entries for local games that have no custom installer or launcher. Universal Game Launcher scans these folders for games and executables using the folder name as the game title.
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

</div>
