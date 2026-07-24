<script lang="ts">
  import { ModeWatcher } from 'mode-watcher';
  import { Toaster } from '$lib/components/ui/sonner';
  import '../app.css';

  let { children } = $props();
</script>

<ModeWatcher />
<Toaster richColors position="top-right" />
{@render children()}
