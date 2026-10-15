<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { t } from "$lib/i18n";
  import { error, games, install, loading, providers } from "$lib/stores";

  onMount(async () => {
    loading.set(true);
    try {
      install.set(await api.detectSteam());
      games.set(await api.loadLibrary());
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      loading.set(false);
    }
  });

  async function refresh() {
    loading.set(true);
    try {
      install.set(await api.detectSteam());
      games.set(await api.loadLibrary());
      error.set(null);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      loading.set(false);
    }
  }

  $: installedGames = $games.filter((game) => game.installed);
  $: shortcuts = $games.filter((game) => game.kind === "shortcut");
</script>

<div class="row-between" style="margin-bottom:1.2rem">
  <div>
    <h1>{$t("dashboard.title")}</h1>
    <p class="muted">{$t("dashboard.subtitle")}</p>
  </div>
  <button class="btn btn-primary" onclick={refresh} disabled={$loading}>
    {$loading ? $t("common.refreshing") : $t("common.refresh")}
  </button>
</div>

<div class="grid stats">
  <div class="glass card stat">
    <div class="muted-sm">{$t("dashboard.totalGames")}</div>
    <div class="stat-value">{$games.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">{$t("dashboard.installedGames")}</div>
    <div class="stat-value">{installedGames.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">{$t("dashboard.shortcutsCount")}</div>
    <div class="stat-value">{shortcuts.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">{$t("storage.steamLibraries")}</div>
    <div class="stat-value">{$install?.libraries.length ?? 0}</div>
  </div>
</div>

<section class="grid two">
  <div class="glass-strong card">
    <h2 class="section-title">Steam installation</h2>
    {#if $install}
      <div class="kv"><span class="muted">Root</span><code>{$install.root}</code></div>
      {#each $install.libraries as library (library.index)}
        <div class="kv"><span class="muted">Library {library.index}</span><code>{library.path}</code></div>
      {/each}
      <h3 class="section-title" style="margin-top:1rem">Accounts</h3>
      {#each $install.accounts as account (account.id3)}
        <div class="kv">
          <span class="muted">{(account.personaName ?? account.accountName) || "Unknown"}</span>
          <code>{account.id64 ?? account.id3}</code>
        </div>
      {/each}
    {:else}
      <p class="muted">No Steam installation detected. Set the Steam folder in <a href="/settings">Settings</a>.</p>
    {/if}
  </div>

  <div class="glass-strong card">
    <h2 class="section-title">Metadata providers</h2>
    <div class="providers">
      {#each $providers as provider (provider.id)}
        <div class="provider">
          <div class="row-between">
            <strong>{provider.label}</strong>
            {#if provider.requiresKey}
              <span class="pill">key required</span>
            {:else}
              <span class="pill pill-dim">keyless</span>
            {/if}
          </div>
          <p class="muted-sm">{provider.description}</p>
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  .stats {
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    margin-bottom: 1.2rem;
    gap: 0.9rem;
  }

  .two {
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    align-items: start;
    gap: 1.1rem;
  }

  .stat {
    position: relative;
    overflow: hidden;
    transition: transform 0.16s ease, border-color 0.16s ease, box-shadow 0.16s ease;
  }

  .stat:hover {
    transform: translateY(-2px);
    border-color: var(--border-strong);
    box-shadow: var(--glass-highlight), 0 10px 28px rgba(0, 0, 0, 0.4);
  }

  .stat-value {
    font-size: 2.1rem;
    font-weight: 750;
    margin-top: 0.2rem;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, #ffffff, var(--accent));
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .kv {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .kv:last-child {
    border-bottom: none;
  }

  .kv code {
    color: var(--accent);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
    text-align: right;
    background: rgba(6, 14, 26, 0.55);
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .providers {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .provider {
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .provider:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }
</style>
