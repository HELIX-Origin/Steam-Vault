<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import OrganizeSection from "$lib/components/settings/OrganizeSection.svelte";
  import "$styles/settings.scss";

  onMount(() => {
    goto("/settings?tab=organize", { replaceState: true });
  });
</script>

<OrganizeSection />
