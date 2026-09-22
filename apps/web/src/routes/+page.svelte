<script lang="ts">
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';

  onMount(() => goto(resolve('/monthly'), { replaceState: true }));
</script>

<svelte:head>
  <meta http-equiv="refresh" content="0;url={resolve('/monthly')}" />
</svelte:head>
