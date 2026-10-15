<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import "$styles/settings.scss";

  onMount(() => {
    goto("/settings?tab=platforms", { replaceState: true });
  });
</script>

<div class="glass" style="padding:2rem;text-align:center">
  <p class="muted">Redirecting to Settings → Platforms &amp; Launchers…</p>
</div>
