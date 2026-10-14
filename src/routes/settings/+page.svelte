<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { error, install, settings } from "$lib/stores";
  import type { Settings } from "$lib/types";

  let form: Settings | null = null;
  let busy = false;
  let saved = false;

  onMount(async () => {
    try {
      const [loaded, detected] = await Promise.all([api.getSettings(), api.detectSteam()]);
      settings.set(loaded);
      form = { ...loaded };
      install.set(detected);
      if (!form.steamRoot && detected) form.steamRoot = detected.root;
      if (!form.accountId3 && detected?.accounts[0]) form.accountId3 = detected.accounts[0].id3;
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
    busy = true;
    try {
      const updated = await api.saveSettings(form);
      settings.set(updated);
      form = { ...updated };
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
    <h1>Settings</h1>
    <p class="muted">Local configuration. API keys stay on this machine and are never committed.</p>
  </div>
  <div class="row">
    {#if saved}<span class="pill">Saved</span>{/if}
    <button class="btn btn-primary" disabled={busy || !form} onclick={save}>{busy ? "Saving…" : "Save"}</button>
  </div>
</div>

{#if form}
  <div class="grid two">
    <section class="glass-strong card">
      <h2 class="section-title">Steam location</h2>
      <div class="field">
        <label for="root">Steam folder</label>
        <input id="root" bind:value={form.steamRoot} placeholder="C:\Program Files (x86)\Steam" />
      </div>
      <button class="btn" onclick={detect}>Auto-detect</button>
      {#if $install}
        <div class="kv"><span class="muted">Libraries</span><span>{$install.libraries.length}</span></div>
        <div class="kv"><span class="muted">Accounts</span><span>{$install.accounts.length}</span></div>
      {/if}
    </section>

    <section class="glass-strong card">
      <h2 class="section-title">Account</h2>
      <div class="field">
        <label for="account">Active Steam account</label>
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
          <label for="cc">Store country</label>
          <input id="cc" bind:value={form.countryCode} placeholder="us" />
        </div>
        <div class="field" style="flex:1">
          <label for="lang">Language</label>
          <input id="lang" bind:value={form.language} placeholder="en" />
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
    padding: 1.1rem 1.2rem;
  }

  .kv {
    display: flex;
    justify-content: space-between;
    padding: 0.35rem 0;
    border-bottom: 1px solid var(--border);
    margin-top: 0.4rem;
  }
</style>
