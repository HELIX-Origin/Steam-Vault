<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
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
    <h1>Dashboard</h1>
    <p class="muted">A clean-room, cross-platform revival of Depressurizer.</p>
  </div>
  <button class="btn btn-primary" onclick={refresh} disabled={$loading}>
    {$loading ? "Refreshing…" : "Refresh"}
  </button>
</div>

<div class="grid stats">
  <div class="glass card stat">
    <div class="muted-sm">Library entries</div>
    <div class="stat-value">{$games.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">Installed</div>
    <div class="stat-value">{installedGames.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">Non-Steam shortcuts</div>
    <div class="stat-value">{shortcuts.length}</div>
  </div>
  <div class="glass card stat">
    <div class="muted-sm">Library folders</div>
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
    grid-template-columns: repeat(4, 1fr);
    margin-bottom: 1rem;
  }

  .two {
    grid-template-columns: 1fr 1fr;
    align-items: start;
  }

  .card {
    padding: 1.1rem 1.2rem;
  }

  .stat-value {
    font-size: 1.9rem;
    font-weight: 700;
    margin-top: 0.15rem;
  }

  .kv {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.3rem 0;
    border-bottom: 1px solid var(--border);
  }

  .kv code {
    color: var(--accent);
    font-size: 0.82rem;
    overflow-wrap: anywhere;
    text-align: right;
  }

  .providers {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .provider {
    padding-bottom: 0.7rem;
    border-bottom: 1px solid var(--border);
  }

  .provider:last-child {
    border-bottom: none;
  }
</style>
