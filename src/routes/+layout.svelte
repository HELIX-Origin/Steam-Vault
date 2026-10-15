<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import WindowControls from "$lib/components/WindowControls.svelte";
  import { isRTL, syncWithSettingsLanguage, t } from "$lib/i18n";
  import { error, providers, settings, steamRunning } from "$lib/stores";

  let { children } = $props();

  const nav = [
    { href: "/", key: "nav.dashboard", icon: "◈" },
    { href: "/library", key: "nav.library", icon: "▤" },
    { href: "/storage", key: "nav.storage", icon: "▥" },
    { href: "/artwork", key: "nav.artwork", icon: "❖" },
    { href: "/collections", key: "nav.collections", icon: "❐" },
    { href: "/platforms", key: "nav.platforms", icon: "⬡" },
    { href: "/organize", key: "nav.organize", icon: "⊞" },
    { href: "/settings", key: "nav.settings", icon: "⚙" }
  ];

  onMount(async () => {
    try {
      providers.set(await api.getProviders());
      const loadedSettings = await api.getSettings();
      settings.set(loadedSettings);
      syncWithSettingsLanguage(loadedSettings?.language);
      steamRunning.set(await api.isSteamRunning());
    } catch (e) {
      error.set(errorMessage(e));
    }
  });
</script>

<div class="app" dir={$isRTL ? "rtl" : "ltr"}>
  <header class="titlebar" data-tauri-drag-region>
    <div class="tb-brand" data-tauri-drag-region>
      <div class="tb-mark" data-tauri-drag-region>
        <img src="/logo.svg" alt="Steam Vault" width="28" height="28" />
      </div>
      <div class="tb-titles" data-tauri-drag-region>
        <span class="tb-title" data-tauri-drag-region>{$t("nav.brandTitle")}</span>
        <span class="tb-sub" data-tauri-drag-region>{$t("nav.brandSubtitle")}</span>
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
            {$t(item.key)}
          </a>
        {/each}
      </nav>

      <div class="status glass">
        <span class:ok-dot={!$steamRunning} class:bad-dot={$steamRunning}></span>
        <span class="muted-sm">{$steamRunning ? $t("status.steamRunning") : $t("status.steamClosed")}</span>
      </div>
    </aside>

    <main class="content">
      {#if $error}
        <div class="error-banner" style="margin-bottom:1rem">
          {$error}
          <button class="btn btn-ghost" style="float:right" onclick={() => error.set(null)}>{$t("status.dismiss")}</button>
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
    height: 48px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px 0 14px;
    user-select: none;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(180deg, rgba(18, 32, 56, 0.65), rgba(9, 18, 32, 0.45));
    backdrop-filter: blur(var(--blur-strong));
    -webkit-backdrop-filter: blur(var(--blur-strong));
    box-shadow: 0 1px 0 0 rgba(255, 255, 255, 0.05);
    z-index: 10;
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
    border-radius: 8px;
    overflow: hidden;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  }

  .tb-mark img {
    width: 100%;
    height: 100%;
    display: block;
  }

  .tb-titles {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .tb-title {
    font-size: 0.92rem;
    font-weight: 650;
    letter-spacing: -0.015em;
    color: var(--text);
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
    padding: 14px 12px 16px;
    gap: 18px;
    box-shadow: var(--glass-highlight), var(--shadow);
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
    padding: 0.6em 0.8em;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    font-weight: 550;
    font-size: 0.88rem;
    border: 1px solid transparent;
    transition: background 0.16s ease, color 0.16s ease, border-color 0.16s ease, transform 0.12s ease;
  }

  .nav-item:hover {
    background: rgba(255, 255, 255, 0.06);
    color: #ffffff;
    transform: translateX(2px);
  }

  :global([dir="rtl"]) .nav-item:hover {
    transform: translateX(-2px);
  }

  .nav-item.active {
    background: linear-gradient(135deg, rgba(56, 189, 248, 0.22), rgba(168, 85, 247, 0.18));
    color: #ffffff;
    border-color: var(--border-glow);
    box-shadow: inset 0 1px 0 0 rgba(255, 255, 255, 0.14), 0 2px 12px rgba(56, 189, 248, 0.16);
  }

  .nav-icon {
    width: 18px;
    text-align: center;
    opacity: 0.85;
    font-size: 0.95rem;
  }

  .status {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0.6em 0.8em;
    border-radius: var(--radius-sm);
    background: var(--panel-subtle);
  }

  .content {
    overflow-y: auto;
    padding: 6px 12px 40px;
  }
</style>
