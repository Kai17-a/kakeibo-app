<script lang="ts">
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';

  const settingsDataUrl = `${resolve('/settings')}?tab=data`;

  onMount(() => goto(settingsDataUrl, { replaceState: true }));
</script>

<svelte:head>
  <meta http-equiv="refresh" content="0;url={settingsDataUrl}" />
</svelte:head>
