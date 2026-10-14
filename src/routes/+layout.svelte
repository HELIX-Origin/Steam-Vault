<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { error, providers, settings, steamRunning } from "$lib/stores";

  let { children } = $props();

  const nav = [
    { href: "/", label: "Dashboard", icon: "◈" },
    { href: "/library", label: "Library", icon: "▤" },
    { href: "/collections", label: "Collections", icon: "❐" },
    { href: "/platforms", label: "Platforms", icon: "⬡" },
    { href: "/organize", label: "Organize", icon: "⊞" },
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

<div class="app">
  <header class="titlebar" data-tauri-drag-region>
    <div class="tb-brand" data-tauri-drag-region>
      <div class="tb-mark" data-tauri-drag-region>SV</div>
      <div class="tb-titles" data-tauri-drag-region>
        <span class="tb-title" data-tauri-drag-region>Steam Vault</span>
        <span class="tb-sub" data-tauri-drag-region>Library organizer</span>
      </div>
    </div>
    <div class="tb-spacer" data-tauri-drag-region></div>
    <WindowControls position="right" />
  </header>

  <div class="shell">
    <aside class="sidebar glass">
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
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .titlebar {
    flex: 0 0 auto;
    height: 46px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px 0 14px;
    user-select: none;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(22, 38, 60, 0.55), rgba(13, 27, 45, 0.35));
    backdrop-filter: blur(var(--blur));
    -webkit-backdrop-filter: blur(var(--blur));
  }

  .tb-brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-left: 4px;
  }

  .tb-mark {
    width: 28px;
    height: 28px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    background: linear-gradient(135deg, rgba(72, 216, 242, 0.32), rgba(176, 108, 255, 0.32));
    border: 1px solid var(--border-strong);
  }

  .tb-titles {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .tb-title {
    font-size: 0.9rem;
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  .tb-sub {
    font-size: 0.74rem;
    color: var(--text-mute);
  }

  .tb-spacer {
    flex: 1;
  }

  .shell {
    display: grid;
    grid-template-columns: 248px 1fr;
    flex: 1;
    min-height: 0;
    padding: 14px;
    gap: 14px;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    padding: 14px 14px 18px;
    gap: 22px;
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
