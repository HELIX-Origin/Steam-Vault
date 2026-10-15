<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import CollectionsSection from "$lib/components/settings/CollectionsSection.svelte";
  import "$styles/settings.scss";

  onMount(() => {
    goto("/settings?tab=collections", { replaceState: true });
  });
</script>

<CollectionsSection />
