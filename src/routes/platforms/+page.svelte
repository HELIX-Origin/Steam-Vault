<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { error, settings } from "$lib/stores";
  import type { Settings } from "$lib/types";
  import PlatformsSection from "$lib/components/settings/PlatformsSection.svelte";
  import "$styles/settings.scss";

  const platformList: { id: string; label: string }[] = [
    { id: "steam", label: "Steam" },
    { id: "gog", label: "GOG" },
    { id: "epic", label: "Epic Games" },
    { id: "itch", label: "itch.io" },
    { id: "ubisoft", label: "Ubisoft Connect" },
    { id: "ea", label: "EA" },
    { id: "battlenet", label: "Battle.net" },
    { id: "amazon", label: "Amazon Games" },
    { id: "rockstar", label: "Rockstar" },
    { id: "xbox", label: "Xbox / Microsoft Store" },
    { id: "other", label: "Other studios" }
  ];

  type SourceRow = { id: string; label: string; enabled: boolean; roots: string[] };
  let sources = $state<SourceRow[]>([]);
  let currentSettings = $state<Settings | null>(null);
  let freeGameNotifications = $state(false);
  let saving = $state(false);
  let saveFeedback = $state<string | null>(null);

  function buildSources(cfg: Settings | null) {
    const savedById = new Map(
      (cfg?.platformSources ?? []).map((source) => [source.id, source])
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

  function addRoot(source: SourceRow) {
    source.roots = [...source.roots, ""];
    sources = [...sources];
    persistSettings();
  }

  function removeRoot(source: SourceRow, index: number) {
    source.roots = source.roots.filter((_, i) => i !== index);
    sources = [...sources];
    persistSettings();
  }

  function setRoot(source: SourceRow, index: number, value: string) {
    source.roots = source.roots.map((root, i) => (i === index ? value : root));
    sources = [...sources];
    persistSettings();
  }

  async function persistSettings() {
    if (!currentSettings) return;
    saving = true;
    try {
      const updated: Settings = {
        ...currentSettings,
        enableFreeGameNotifications: freeGameNotifications,
        platformSources: sources.map((s) => ({
          id: s.id,
          enabled: s.enabled,
          roots: s.roots.filter((r) => r.trim().length > 0)
        }))
      };
      await api.saveSettings(updated);
      settings.set(updated);
      currentSettings = updated;
      saveFeedback = "Settings saved";
      setTimeout(() => {
        saveFeedback = null;
      }, 2500);
    } catch (e) {
      error.set(errorMessage(e));
    } finally {
      saving = false;
    }
  }

  onMount(async () => {
    try {
      if ($settings) {
        currentSettings = $settings;
        freeGameNotifications = Boolean($settings.enableFreeGameNotifications);
        buildSources($settings);
      } else {
        const loaded = await api.getSettings();
        currentSettings = loaded;
        freeGameNotifications = Boolean(loaded.enableFreeGameNotifications);
        settings.set(loaded);
        buildSources(loaded);
      }
    } catch (e) {
      error.set(errorMessage(e));
    }
  });
</script>

<div class="platforms-page">
  <div class="page-head glass-strong card" style="margin-bottom:1.25rem;padding:1.25rem 1.5rem">
    <div class="row-between" style="align-items:center">
      <div>
        <h1 style="font-size:1.45rem;font-weight:700;margin:0 0 0.35rem">Platforms &amp; Native Launchers</h1>
        <p class="muted-sm" style="margin:0">
          Direct integration with official game launchers. Games are organized in your unified library and launched directly through official clients.
        </p>
      </div>
      <div class="row" style="gap:0.75rem;align-items:center">
        {#if saveFeedback}
          <span class="pill pill-ok" style="font-size:0.75rem">{saveFeedback}</span>
        {/if}
        <button class="btn btn-ghost" onclick={persistSettings} disabled={saving}>
          {saving ? "Saving…" : "Save Changes"}
        </button>
      </div>
    </div>
  </div>

  <PlatformsSection
    bind:sources
    bind:freeGameNotifications
    {addRoot}
    {removeRoot}
    {setRoot}
  />
</div>

<style>
  .platforms-page {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
