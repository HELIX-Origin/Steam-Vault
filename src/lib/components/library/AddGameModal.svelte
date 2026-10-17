<script lang="ts">
  import { api, errorMessage } from "$lib/api";
  import { games, selectedGameId } from "$lib/stores";
  import type { PlatformGame } from "$lib/types";

  interface Props {
    onclose: () => void;
    onadded?: (addedAppId?: number) => void;
  }

  let { onclose, onadded }: Props = $props();

  let gameName = $state("");
  let exePath = $state("");
  let workingDir = $state("");
  let launchArgs = $state("");
  let coverImage = $state("");

  let submitting = $state(false);
  let feedback = $state<{ success: boolean; message: string } | null>(null);

  function deriveNameFromPath(path: string): string {
    if (!path) return "";
    const clean = path.replace(/[\\/]+$/, "");
    const base = clean.split(/[\\/]/).pop() || "";
    // Strip common executable extensions
    const stem = base.replace(/\.(exe|bat|cmd|sh|bin|app|jar|py|love)$/i, "");
    // Clean up underscores and hyphens if title-like
    return stem.replace(/[_-]+/g, " ").trim();
  }

  async function handleBrowseExe() {
    try {
      const picked = await api.pickGameExecutable();
      if (picked) {
        exePath = picked;
        if (!gameName.trim()) {
          gameName = deriveNameFromPath(picked);
        }
        if (!workingDir.trim()) {
          const lastSlash = Math.max(picked.lastIndexOf("/"), picked.lastIndexOf("\\"));
          if (lastSlash > 0) {
            workingDir = picked.slice(0, lastSlash);
          }
        }
      }
    } catch (e) {
      feedback = { success: false, message: `Could not open file picker: ${errorMessage(e)}` };
    }
  }

  async function handleBrowseFolder() {
    try {
      const picked = await api.pickGameDirectory();
      if (picked) {
        workingDir = picked;
      }
    } catch (e) {
      feedback = { success: false, message: `Could not open folder picker: ${errorMessage(e)}` };
    }
  }

  async function handleBrowseArtwork() {
    try {
      const picked = await api.pickGameArtwork();
      if (picked) {
        coverImage = picked;
      }
    } catch (e) {
      feedback = { success: false, message: `Could not open image picker: ${errorMessage(e)}` };
    }
  }

  async function handleAddGame() {
    const trimmedName = gameName.trim();
    const trimmedExe = exePath.trim();

    if (!trimmedName) {
      feedback = { success: false, message: "Please specify a name for the game." };
      return;
    }
    if (!trimmedExe) {
      feedback = { success: false, message: "Please select or enter the game executable path." };
      return;
    }

    submitting = true;
    feedback = null;

    try {
      const payload: PlatformGame = {
        name: trimmedName,
        platform: "standalone",
        platformId: null,
        installDir: workingDir.trim() || null,
        exe: trimmedExe,
        sizeOnDisk: 0,
        launchUri: launchArgs.trim() || null,
        source: "Manual Standalone Entry",
        installed: true
      };

      const result = await api.importPlatformGames([payload]);
      const addedGame = result.added?.[0];

      if (addedGame) {
        // If custom artwork was provided, cache it
        if (coverImage.trim()) {
          try {
            await api.cacheArtworkImage(addedGame.appId, "capsule", coverImage.trim());
          } catch {
            // Non-critical, continue
          }
        }

        // Refresh active library
        const updatedLib = await api.loadLibrary(undefined, true);
        games.set(updatedLib);
        selectedGameId.set(addedGame.appId);

        feedback = { success: true, message: `"${trimmedName}" added to library!` };
        setTimeout(() => {
          onadded?.(addedGame.appId);
          onclose();
        }, 700);
      } else {
        feedback = {
          success: false,
          message: result.skipped?.length
            ? `Game skipped: ${result.skipped.join(", ")}`
            : "Failed to add game to library."
        };
      }
    } catch (e) {
      feedback = { success: false, message: errorMessage(e) };
    } finally {
      submitting = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="modal-backdrop" onclick={(e) => { if (e.target === e.currentTarget) onclose(); }} role="presentation">
  <div class="modal-dialog glass-strong add-game-dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
    <div class="modal-header">
      <div class="row" style="align-items: center; gap: 0.6rem;">
        <span class="dialog-icon">⚙</span>
        <div>
          <h2 class="dialog-title">Add Local Game Manually</h2>
          <p class="dialog-subtitle">Import standalone executables, portable games, or custom DRM-free titles</p>
        </div>
      </div>
      <button class="btn mini btn-ghost close-btn" onclick={onclose} title="Close">✕</button>
    </div>

    {#if feedback}
      <div
        class="feedback-banner"
        class:banner-success={feedback.success}
        class:banner-error={!feedback.success}
      >
        <span>{feedback.success ? "✓" : "✗"}</span>
        <p>{feedback.message}</p>
      </div>
    {/if}

    <form onsubmit={(e) => { e.preventDefault(); handleAddGame(); }} class="dialog-body">
      <!-- Game Name -->
      <div class="form-group">
        <label for="manual-game-name" class="form-label">
          Game Title <span class="required-star">*</span>
        </label>
        <input
          id="manual-game-name"
          type="text"
          class="input-text"
          bind:value={gameName}
          placeholder="e.g. Celeste, Half-Life 2, OpenTTD"
          required
        />
        <span class="muted-xs">The display name for the game in your library and showcase.</span>
      </div>

      <!-- Executable Path -->
      <div class="form-group">
        <label for="manual-game-exe" class="form-label">
          Executable Path <span class="required-star">*</span>
        </label>
        <div class="input-with-button">
          <input
            id="manual-game-exe"
            type="text"
            class="input-text"
            bind:value={exePath}
            placeholder="e.g. C:\Games\MyGame\game.exe"
            required
          />
          <button type="button" class="btn btn-secondary picker-btn" onclick={handleBrowseExe} title="Browse for executable file">
            📁 Browse…
          </button>
        </div>
        <span class="muted-xs">Supports .exe, .bat, .cmd, .sh, .bin, .jar, and script launchers.</span>
      </div>

      <!-- Working Directory -->
      <div class="form-group">
        <label for="manual-game-dir" class="form-label">
          Start / Working Directory <span class="muted-xs">(Optional)</span>
        </label>
        <div class="input-with-button">
          <input
            id="manual-game-dir"
            type="text"
            class="input-text"
            bind:value={workingDir}
            placeholder="Defaults to executable parent folder"
          />
          <button type="button" class="btn btn-secondary picker-btn" onclick={handleBrowseFolder} title="Browse for folder">
            📂 Folder…
          </button>
        </div>
      </div>

      <!-- Launch Arguments -->
      <div class="form-group">
        <label for="manual-game-args" class="form-label">
          Launch Arguments / Options <span class="muted-xs">(Optional)</span>
        </label>
        <input
          id="manual-game-args"
          type="text"
          class="input-text"
          bind:value={launchArgs}
          placeholder="e.g. -windowed -novid -fullscreen"
        />
      </div>

      <!-- Custom Cover Artwork -->
      <div class="form-group">
        <label for="manual-game-cover" class="form-label">
          Cover Artwork Image <span class="muted-xs">(Optional)</span>
        </label>
        <div class="input-with-button">
          <input
            id="manual-game-cover"
            type="text"
            class="input-text"
            bind:value={coverImage}
            placeholder="Paste image URL or pick local image file"
          />
          <button type="button" class="btn btn-secondary picker-btn" onclick={handleBrowseArtwork} title="Browse for cover image">
            🖼️ Image…
          </button>
        </div>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-ghost" onclick={onclose} disabled={submitting}>
          Cancel
        </button>
        <button type="submit" class="btn btn-primary submit-btn" disabled={submitting}>
          {submitting ? "Adding Game…" : "➕ Add to Library"}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(4, 2, 10, 0.78);
    backdrop-filter: blur(14px);
    animation: fadeIn 0.16s ease-out;
  }

  .add-game-dialog {
    width: 90vw;
    max-width: 580px;
    background: #0f0923;
    border: 1px solid rgba(168, 85, 247, 0.35);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.75), 0 0 32px rgba(168, 85, 247, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.1rem 1.4rem;
    border-bottom: 1px solid rgba(168, 85, 247, 0.18);
    background: rgba(18, 10, 38, 0.9);
  }

  .dialog-icon {
    font-size: 1.4rem;
    color: #c084fc;
    filter: drop-shadow(0 0 8px rgba(192, 132, 252, 0.5));
  }

  .dialog-title {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
    color: #ffffff;
    letter-spacing: -0.01em;
  }

  .dialog-subtitle {
    margin: 0.15rem 0 0;
    font-size: 0.74rem;
    color: #a78bfa;
  }

  .close-btn {
    font-size: 1.1rem;
    cursor: pointer;
    color: var(--text-mute);
  }

  .close-btn:hover {
    color: #ffffff;
  }

  .dialog-body {
    padding: 1.25rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .form-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: #e2e8f0;
  }

  .required-star {
    color: #f43f5e;
  }

  .input-with-button {
    display: flex;
    gap: 0.4rem;
  }

  .input-text {
    flex: 1;
    background: rgba(15, 8, 30, 0.85);
    border: 1px solid rgba(168, 85, 247, 0.25);
    border-radius: var(--radius-sm, 6px);
    padding: 7px 10px;
    font-size: 0.84rem;
    color: #ffffff;
    outline: none;
    transition: all 0.15s ease;
  }

  .input-text:focus {
    border-color: rgba(192, 132, 252, 0.8);
    box-shadow: 0 0 10px rgba(168, 85, 247, 0.35);
  }

  .picker-btn {
    white-space: nowrap;
    padding: 0 12px;
    font-size: 0.8rem;
    font-weight: 600;
    border-radius: var(--radius-sm, 6px);
    background: rgba(168, 85, 247, 0.18);
    border: 1px solid rgba(168, 85, 247, 0.4);
    color: #e9d5ff;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .picker-btn:hover {
    background: rgba(168, 85, 247, 0.35);
    color: #ffffff;
    border-color: rgba(192, 132, 252, 0.8);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(168, 85, 247, 0.15);
    margin-top: 0.5rem;
  }

  .submit-btn {
    background: linear-gradient(135deg, #7c3aed, #a855f7);
    color: #ffffff;
    font-weight: 600;
    padding: 7px 18px;
    border-radius: var(--radius-sm, 6px);
    border: none;
    box-shadow: 0 2px 10px rgba(124, 58, 237, 0.4);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .submit-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(168, 85, 247, 0.6);
  }

  .feedback-banner {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.6rem 1.4rem;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .banner-success {
    background: rgba(16, 185, 129, 0.15);
    border-bottom: 1px solid rgba(16, 185, 129, 0.35);
    color: #6ee7b7;
  }

  .banner-error {
    background: rgba(244, 63, 94, 0.15);
    border-bottom: 1px solid rgba(244, 63, 94, 0.35);
    color: #fda4af;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
