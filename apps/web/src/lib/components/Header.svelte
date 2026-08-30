<script lang="ts">
  import { resolve } from '$app/paths';
  import CircleDollarSignIcon from '@lucide/svelte/icons/circle-dollar-sign';
  import MoonIcon from '@lucide/svelte/icons/moon';
  import ReceiptTextIcon from '@lucide/svelte/icons/receipt-text';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import SunIcon from '@lucide/svelte/icons/sun';
  import { Button } from '$lib/components/ui/button';
  import { toggleMode } from 'mode-watcher';

  interface Props {
    onTransaction?: () => void;
    onRecurring?: () => void;
    onRecurringIncome?: () => void;
  }
  let { onTransaction, onRecurring, onRecurringIncome }: Props = $props();
</script>

<header class="border-b bg-background/95">
  <div class="mx-auto flex max-w-7xl items-center justify-between px-5 py-4 lg:px-10">
    <a class="flex items-center gap-3" href={resolve('/')} aria-label="Kakeibo ホーム">
      <span
        class="grid size-10 place-items-center rounded-lg bg-primary text-lg font-black text-primary-foreground"
      >
        K
      </span>
      <strong class="hidden font-serif text-xl tracking-tight sm:block">Kakeibo</strong>
    </a>
    <div class="flex items-center gap-2">
      <Button variant="outline" size="icon" onclick={toggleMode} aria-label="テーマを切り替え">
        <SunIcon class="dark:hidden" />
        <MoonIcon class="hidden dark:block" />
      </Button>
      <Button variant="outline" size="icon" href="/settings" aria-label="設定">
        <SettingsIcon />
      </Button>
      {#if onRecurring && onTransaction}
        <Button variant="outline" onclick={onRecurring} aria-label="固定費">
          <ReceiptTextIcon data-icon="inline-start" />
          <span class="hidden sm:inline">固定費</span>
        </Button>
        {#if onRecurringIncome}
          <Button variant="outline" onclick={onRecurringIncome} aria-label="定期収入">
            <CircleDollarSignIcon data-icon="inline-start" />
            <span class="hidden lg:inline">定期収入</span>
          </Button>
        {/if}
        <Button onclick={onTransaction} aria-label="収支を登録">
          <CircleDollarSignIcon data-icon="inline-start" />
          <span class="hidden sm:inline">収支を登録</span>
        </Button>
      {/if}
    </div>
  </div>
</header>
