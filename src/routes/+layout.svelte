<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { error, providers, settings, steamRunning } from "$lib/stores";

  let { children } = $props();

  const nav = [
    { href: "/", label: "Dashboard", icon: "◈" },
    { href: "/library", label: "Library", icon: "▤" },
    { href: "/collections", label: "Collections", icon: "❐" },
    { href: "/settings", label: "Settings", icon: "⚙" }
  ];

  onMount(async () => {
    try {
      providers.set(await api.getProviders());
      settings.set(await api.getSettings());
      steamRunning.set(await api.isSteamRunning());
    } catch (e) {
      error.set(errorMessage(e));
    }
  });
</script>

<div class="shell">
  <aside class="sidebar glass">
    <div class="brand">
      <div class="brand-mark">SV</div>
      <div>
        <div class="brand-name">Steam Vault</div>
        <div class="muted-sm">Library organizer</div>
      </div>
    </div>

    <nav>
      {#each nav as item (item.href)}
        <a class="nav-item" class:active={page.url.pathname === item.href} href={item.href}>
          <span class="nav-icon">{item.icon}</span>
          {item.label}
        </a>
      {/each}
    </nav>

    <div class="status glass">
      <span class:ok-dot={!$steamRunning} class:bad-dot={$steamRunning}></span>
      <span class="muted-sm">{$steamRunning ? "Steam is running" : "Steam is closed"}</span>
    </div>
  </aside>

  <main class="content">
    {#if $error}
      <div class="error-banner" style="margin-bottom:1rem">
        {$error}
        <button class="btn btn-ghost" style="float:right" onclick={() => error.set(null)}>Dismiss</button>
      </div>
    {/if}
    {@render children?.()}
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 248px 1fr;
    height: 100vh;
    padding: 14px;
    gap: 14px;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    padding: 18px 14px;
    gap: 22px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 6px;
  }

  .brand-mark {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    font-weight: 700;
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.3), rgba(176, 108, 255, 0.3));
    border: 1px solid var(--border-strong);
  }

  .brand-name {
    font-weight: 650;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0.6em 0.75em;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    font-weight: 550;
    transition: background 0.15s ease, color 0.15s ease;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text);
  }

  .nav-item.active {
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.18), rgba(176, 108, 255, 0.16));
    color: var(--text);
    border: 1px solid var(--border-strong);
  }

  .nav-icon {
    width: 18px;
    text-align: center;
    opacity: 0.85;
  }

  .status {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0.6em 0.8em;
    border-radius: var(--radius-sm);
  }

  .content {
    overflow-y: auto;
    padding: 8px 10px 40px;
  }
</style>
