<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorMessage } from "$lib/api";
  import { pluginEngine } from "$lib/plugins/engine";
  import type { PluginInfo, PluginSettingField, PluginSettingsConfig } from "$lib/types";

  interface Props {
    initialPluginId?: string | null;
    onclose: () => void;
  }

  let { initialPluginId = null, onclose }: Props = $props();

  let plugins = $state<PluginInfo[]>([]);
  let loading = $state(true);
  let activePluginId = $state<string>("");
  let formValues = $state<Record<string, any>>({});
  let showPasswordMap = $state<Record<string, boolean>>({});

  let testing = $state(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);

  let saving = $state(false);
  let saveSuccess = $state(false);
  let errorMsg = $state<string | null>(null);

  // Filter plugins that are enabled AND declare settings fields
  let configurablePlugins = $derived.by(() => {
    return plugins.filter((p) => {
      const schema = p.settings || pluginEngine.getPluginSettingsSchema(p.id);
      return p.enabled && schema && schema.fields && schema.fields.length > 0;
    });
  });

  let activePlugin = $derived(
    configurablePlugins.find((p) => p.id === activePluginId) || configurablePlugins[0] || null
  );

  let activeSchema = $derived<PluginSettingsConfig | null>(
    activePlugin?.settings || (activePlugin ? pluginEngine.getPluginSettingsSchema(activePlugin.id) : null)
  );

  onMount(async () => {
    loading = true;
    try {
      const backendList = await api.getInstalledPlugins();
      plugins = pluginEngine.mergePluginInfo(backendList);

      const candidateList = plugins.filter((p) => {
        const schema = p.settings || pluginEngine.getPluginSettingsSchema(p.id);
        return p.enabled && schema && schema.fields && schema.fields.length > 0;
      });

      if (initialPluginId && candidateList.some((p) => p.id === initialPluginId)) {
        activePluginId = initialPluginId;
      } else if (candidateList.length > 0) {
        activePluginId = candidateList[0].id;
      }

      if (activePluginId) {
        await loadActivePluginForm(activePluginId);
      }
    } catch (e) {
      errorMsg = errorMessage(e);
    } finally {
      loading = false;
    }
  });

  async function selectPlugin(id: string) {
    if (activePluginId === id) return;
    activePluginId = id;
    testResult = null;
    saveSuccess = false;
    await loadActivePluginForm(id);
  }

  async function loadActivePluginForm(id: string) {
    try {
      formValues = await pluginEngine.loadPluginConfig(id);
    } catch (e) {
      errorMsg = errorMessage(e);
    }
  }

  function togglePassword(key: string) {
    showPasswordMap[key] = !showPasswordMap[key];
    showPasswordMap = { ...showPasswordMap };
  }

  async function handleTestConnection() {
    if (!activePlugin) return;
    testing = true;
    testResult = null;
    try {
      testResult = await pluginEngine.testPluginConnection(activePlugin.id, formValues);
    } catch (e) {
      testResult = { success: false, message: errorMessage(e) };
    } finally {
      testing = false;
    }
  }

  async function handleSave() {
    if (!activePlugin) return;
    saving = true;
    errorMsg = null;
    try {
      await pluginEngine.savePluginConfig(activePlugin.id, formValues);
      saveSuccess = true;
      setTimeout(() => (saveSuccess = false), 3000);
    } catch (e) {
      errorMsg = errorMessage(e);
    } finally {
      saving = false;
    }
  }

  async function openHelpUrl(url: string) {
    try {
      await api.openExternalUrl(url);
    } catch {
      window.open(url, "_blank");
    }
  }

  function getFieldHelpLabel(field: PluginSettingField): string {
    if (field.helpLabel) return field.helpLabel;
    const keyLower = field.key.toLowerCase();
    const labelLower = field.label.toLowerCase();
    const isApiKey =
      keyLower.includes("key") ||
      keyLower.includes("secret") ||
      keyLower.includes("token") ||
      labelLower.includes("key") ||
      labelLower.includes("secret") ||
      labelLower.includes("token");

    if (isApiKey) {
      return "Get Key ↗";
    }
    return "Documentation ↗";
  }
</script>

<div
  class="dialog-backdrop"
  role="dialog"
  aria-modal="true"
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onclose();
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") onclose();
  }}
>
  <div class="dialog-window plugin-settings-subwindow glass-strong">
    <!-- Window Header -->
    <div class="subwindow-header">
      <div class="row" style="align-items: center; gap: 0.6rem;">
        <span class="subwindow-icon">🧩</span>
        <div>
          <h2 class="subwindow-title">Plugin Settings &amp; Credentials</h2>
          <p class="subwindow-subtitle">
            Configure credentials, API keys, and options for enabled launcher plugins
          </p>
        </div>
      </div>
      <button class="btn mini btn-ghost close-btn" onclick={onclose} title="Close Settings Window">
        ✕
      </button>
    </div>

    <!-- Window Body with Tabbed Layout -->
    <div class="subwindow-body">
      {#if loading}
        <div class="loading-state">
          <p class="muted">Loading plugin configurations…</p>
        </div>
      {:else if configurablePlugins.length === 0}
        <div class="empty-state">
          <p class="muted">No enabled plugins with configurable settings were found.</p>
          <p class="muted-xs">Enable plugins from the Plugins tab to configure their settings.</p>
        </div>
      {:else}
        <!-- Left Column: Plugin Tabs -->
        <nav class="plugin-tab-rail" aria-label="Plugin Settings Tabs">
          {#each configurablePlugins as plugin (plugin.id)}
            <button
              class="plugin-tab-item"
              class:active={plugin.id === activePluginId}
              onclick={() => selectPlugin(plugin.id)}
            >
              <div class="tab-indicator"></div>
              <div class="tab-content">
                <strong class="tab-title">{plugin.name}</strong>
                <span class="tab-meta muted-xs">
                  <span class="pill pill-dim" style="font-size: 0.65rem; text-transform: uppercase;">
                    {plugin.pluginType}
                  </span>
                  v{plugin.version}
                </span>
              </div>
            </button>
          {/each}
        </nav>

        <!-- Right Column: Active Plugin Settings Form -->
        <div class="plugin-settings-pane">
          {#if activePlugin}
            <div class="plugin-pane-header">
              <div class="row-between" style="align-items: flex-start;">
                <div>
                  <h3 class="pane-title">{activePlugin.name}</h3>
                  <p class="pane-meta muted-xs">
                    By {activePlugin.author} • Version {activePlugin.version}
                    {#if activePlugin.homepage}
                      • <a
                          href={activePlugin.homepage}
                          onclick={(e) => {
                            e.preventDefault();
                            if (activePlugin?.homepage) openHelpUrl(activePlugin.homepage);
                          }}
                          class="accent-link"
                        >
                          Documentation ↗
                        </a>
                    {/if}
                  </p>
                </div>
                <span class="pill pill-accent" style="font-size: 0.7rem; text-transform: uppercase;">
                  {activePlugin.pluginType}
                </span>
              </div>
              {#if activePlugin.description}
                <p class="pane-desc muted-sm">{activePlugin.description}</p>
              {/if}
            </div>

            <!-- Settings Fields Form -->
            <div class="plugin-fields-form">
              {#if activeSchema && activeSchema.fields && activeSchema.fields.length > 0}
                {#each activeSchema.fields as field (field.key)}
                  <div class="field-row">
                    <div class="field-label-group">
                      <label for="cfg-{field.key}">
                        {field.label}
                        {#if field.required}
                          <span class="required-star">*</span>
                        {/if}
                      </label>
                      {#if field.helpUrl}
                        <button
                          type="button"
                          class="btn-text-link muted-xs"
                          onclick={() => openHelpUrl(field.helpUrl!)}
                          title="Open developer console or documentation"
                        >
                          {getFieldHelpLabel(field)}
                        </button>
                      {/if}
                    </div>

                    {#if field.description}
                      <p class="field-desc muted-xs">{field.description}</p>
                    {/if}

                    <div class="field-input-wrapper">
                      {#if field.type === "password"}
                        <div class="password-input-group">
                          <input
                            id="cfg-{field.key}"
                            type={showPasswordMap[field.key] ? "text" : "password"}
                            placeholder={field.placeholder ?? "Enter secret key…"}
                            bind:value={formValues[field.key]}
                            autocomplete="off"
                            spellcheck="false"
                          />
                          <button
                            type="button"
                            class="btn-icon-eye"
                            onclick={() => togglePassword(field.key)}
                            title={showPasswordMap[field.key] ? "Hide Secret" : "Show Secret"}
                          >
                            {showPasswordMap[field.key] ? "👁️" : "🔒"}
                          </button>
                        </div>
                      {:else if field.type === "boolean"}
                        <label class="toggle-switch-label">
                          <input
                            type="checkbox"
                            id="cfg-{field.key}"
                            bind:checked={formValues[field.key]}
                          />
                          <span class="toggle-slider"></span>
                          <span class="toggle-text muted-xs">
                            {formValues[field.key] ? "Enabled" : "Disabled"}
                          </span>
                        </label>
                      {:else if field.type === "select" && field.options}
                        <select id="cfg-{field.key}" bind:value={formValues[field.key]}>
                          {#each field.options as opt}
                            <option value={opt.value}>{opt.label}</option>
                          {/each}
                        </select>
                      {:else if field.type === "number"}
                        <input
                          id="cfg-{field.key}"
                          type="number"
                          placeholder={field.placeholder ?? ""}
                          bind:value={formValues[field.key]}
                        />
                      {:else}
                        <input
                          id="cfg-{field.key}"
                          type="text"
                          placeholder={field.placeholder ?? ""}
                          bind:value={formValues[field.key]}
                        />
                      {/if}
                    </div>
                  </div>
                {/each}
              {:else}
                <p class="muted-xs">This plugin does not declare custom configuration fields.</p>
              {/if}
            </div>

            <!-- Test Connection Result Bar -->
            {#if testResult}
              <div
                class="test-result-banner"
                class:banner-success={testResult.success}
                class:banner-error={!testResult.success}
              >
                <span>{testResult.success ? "✓" : "✗"}</span>
                <p>{testResult.message}</p>
              </div>
            {/if}

            {#if errorMsg}
              <div class="test-result-banner banner-error">
                <span>✗</span>
                <p>{errorMsg}</p>
              </div>
            {/if}

            <!-- Pane Action Footer -->
            <div class="plugin-pane-footer">
              <div class="row" style="align-items: center; gap: 0.5rem;">
                <button
                  type="button"
                  class="btn mini btn-ghost"
                  onclick={handleTestConnection}
                  disabled={testing}
                  title="Test authentication or API connectivity"
                >
                  {testing ? "Testing…" : "🔌 Test Connection"}
                </button>
              </div>

              <div class="row" style="align-items: center; gap: 0.6rem;">
                {#if saveSuccess}
                  <span class="pill pill-ok">✓ Saved</span>
                {/if}
                <button type="button" class="btn mini" onclick={onclose}>
                  Close
                </button>
                <button
                  type="button"
                  class="btn mini btn-primary"
                  onclick={handleSave}
                  disabled={saving}
                >
                  {saving ? "Saving…" : "💾 Save Settings"}
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 1.5rem;
  }

  .plugin-settings-subwindow {
    width: 880px;
    max-width: 95vw;
    height: 600px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(18, 22, 32, 0.94);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
    overflow: hidden;
  }

  .subwindow-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.1rem 1.4rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.02);
  }

  .subwindow-icon {
    font-size: 1.4rem;
  }

  .subwindow-title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    color: #fff;
  }

  .subwindow-subtitle {
    margin: 0.15rem 0 0 0;
    font-size: 0.78rem;
    color: var(--muted, #8e99a8);
  }

  .close-btn {
    font-size: 0.9rem;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
  }

  .subwindow-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .loading-state,
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
  }

  /* Left Tab Rail */
  .plugin-tab-rail {
    width: 280px;
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.18);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 0.5rem;
    gap: 0.25rem;
  }

  .plugin-tab-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem 0.85rem;
    border-radius: 8px;
    border: 1px solid transparent;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .plugin-tab-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .plugin-tab-item.active {
    background: rgba(255, 255, 255, 0.09);
    border-color: rgba(255, 255, 255, 0.16);
  }

  .tab-indicator {
    width: 4px;
    height: 20px;
    border-radius: 2px;
    background: transparent;
  }

  .plugin-tab-item.active .tab-indicator {
    background: var(--accent, #6366f1);
  }

  .tab-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .tab-title {
    font-size: 0.84rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #fff;
  }

  .tab-meta {
    margin-top: 0.2rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  /* Right Pane */
  .plugin-settings-pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 1.25rem 1.5rem;
  }

  .plugin-pane-header {
    margin-bottom: 1.25rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .pane-title {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .pane-meta {
    margin: 0.25rem 0 0 0;
  }

  .pane-desc {
    margin: 0.5rem 0 0 0;
    line-height: 1.45;
  }

  .accent-link {
    color: var(--accent, #6366f1);
    text-decoration: none;
  }

  .accent-link:hover {
    text-decoration: underline;
  }

  /* Fields Form */
  .plugin-fields-form {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
  }

  .field-row {
    display: flex;
    flex-direction: column;
  }

  .field-label-group {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.25rem;
  }

  .field-label-group label {
    font-size: 0.85rem;
    font-weight: 500;
    color: #eee;
  }

  .required-star {
    color: #ff6b6b;
    margin-left: 0.2rem;
  }

  .btn-text-link {
    background: transparent;
    border: none;
    color: var(--accent, #6366f1);
    cursor: pointer;
    padding: 0;
    font-size: 0.75rem;
  }

  .btn-text-link:hover {
    text-decoration: underline;
  }

  .field-desc {
    margin: 0 0 0.4rem 0;
    line-height: 1.35;
  }

  .password-input-group {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .password-input-group input {
    flex: 1;
  }

  .btn-icon-eye {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    font-size: 0.9rem;
    cursor: pointer;
    color: inherit;
  }

  .btn-icon-eye:hover {
    background: rgba(255, 255, 255, 0.14);
  }

  /* Toggle Switch */
  .toggle-switch-label {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    cursor: pointer;
    user-select: none;
  }

  .toggle-switch-label input {
    display: none;
  }

  .toggle-slider {
    width: 36px;
    height: 20px;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    position: relative;
    transition: background 0.2s ease;
  }

  .toggle-slider::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s ease;
  }

  .toggle-switch-label input:checked + .toggle-slider {
    background: var(--accent, #6366f1);
  }

  .toggle-switch-label input:checked + .toggle-slider::after {
    transform: translateX(16px);
  }

  /* Result Banner */
  .test-result-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.65rem 0.9rem;
    border-radius: 8px;
    font-size: 0.8rem;
    margin-top: 0.75rem;
  }

  .banner-success {
    background: rgba(46, 213, 115, 0.12);
    border: 1px solid rgba(46, 213, 115, 0.35);
    color: #2ed573;
  }

  .banner-error {
    background: rgba(255, 71, 87, 0.12);
    border: 1px solid rgba(255, 71, 87, 0.35);
    color: #ff4757;
  }

  .test-result-banner p {
    margin: 0;
    line-height: 1.35;
  }

  /* Pane Footer */
  .plugin-pane-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 1.5rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }
</style>
