<script lang="ts">
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error, platformScan } from "$lib/stores";
  import type { EmulatorConfig, Platform, PlatformGame } from "$lib/types";

  type SourceRow = { id: string; label: string; enabled: boolean; roots: string[] };

  let {
    sources = $bindable(),
    emulatorConfigs = $bindable(),
    popularEmulators,
    scaffoldPath = $bindable(),
    scaffolding = $bindable(),
    scaffoldMsg = $bindable(),
    scaffoldEsDe,
    addEmulatorConfig,
    removeEmulatorConfig,
    addEmulatorRomDir,
    removeEmulatorRomDir,
    setEmulatorRomDir,
    addRoot,
    removeRoot,
    setRoot
  }: {
    sources: SourceRow[];
    emulatorConfigs: EmulatorConfig[];
    popularEmulators: { name: string; exe: string }[];
    scaffoldPath: string;
    scaffolding: boolean;
    scaffoldMsg: string | null;
    scaffoldEsDe: () => Promise<void>;
    addEmulatorConfig: () => void;
    removeEmulatorConfig: (index: number) => void;
    addEmulatorRomDir: (emu: EmulatorConfig) => void;
    removeEmulatorRomDir: (emu: EmulatorConfig, dirIndex: number) => void;
    setEmulatorRomDir: (emu: EmulatorConfig, dirIndex: number, val: string) => void;
    addRoot: (source: SourceRow) => void;
    removeRoot: (source: SourceRow, index: number) => void;
    setRoot: (source: SourceRow, index: number, value: string) => void;
  } = $props();

  const labels: Record<Platform, string> = {
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
    emulator: "Emulators & ROMs",
    other: "Other"
  };

  let selected = $state(new Set<string>());
  let filter = $state<Platform | "all">("all");
  let busy = $state(false);
  let imported = $state<{ added: number; skipped: string[] } | null>(null);
  let includeOtherDrives = $state(false);

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
      <span class="pill pill-dim">Local Storefronts</span>
    </div>
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
              <span class="name">{status.label}</span>
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
          Import {selected.size || ""} selected to Steam
        </button>
      </div>

      <div class="glass platform-list">
        {#each shown as game (key(game))}
          <button class="row platform-game" class:selected={selected.has(key(game))} onclick={() => toggle(game)}>
            <span class="platform-check">{selected.has(key(game)) ? "✓" : ""}</span>
            <span class="platform-name">
              {game.name}
              <span class="pill pill-accent2">{labels[game.platform]}</span>
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
          Steam Vault reads launcher manifests and local registries without launching background processes.
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

  <!-- EmulationStation (ES-DE) & ROM Directories -->
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
</div>
