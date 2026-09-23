<script lang="ts">
  import LedgerPage from '$lib/features/ledger/LedgerPage.svelte';

  let { children } = $props();
</script>

<LedgerPage />
{@render children()}
