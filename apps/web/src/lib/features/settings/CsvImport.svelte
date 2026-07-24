<script lang="ts">
  import UploadIcon from '@lucide/svelte/icons/upload';
  import { toast } from 'svelte-sonner';
  import { buttonVariants } from '$lib/components/ui/button';
  import { api } from '$lib/api';
  import type { ImportResult } from '$lib/types';
  import { cn } from '$lib/utils';

  interface Props {
    kind: 'expense' | 'income';
    onimported?: () => void;
  }
  let { kind, onimported }: Props = $props();

  let importing = $state(false);
  let error = $state('');

  const label = $derived(kind === 'expense' ? '支出' : '収入');

  async function importFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    importing = true;
    error = '';
    try {
      const csv = await file.text();
      const result =
        kind === 'expense' ? await api.importExpenses(csv) : await api.importIncomes(csv);
      toast.success(successMessage(result));
      onimported?.();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'インポートできませんでした。';
    } finally {
      importing = false;
    }
  }

  function successMessage(result: ImportResult) {
    const created = [
      ...result.created_categories.map((name) => `カテゴリ「${name}」`),
      ...result.created_payment_methods.map((name) => `支払方法「${name}」`),
    ];
    const suffix = created.length > 0 ? `${created.join('、')}を追加しました。` : '';
    return `${result.imported}件の${label}をインポートしました。${suffix}`;
  }
</script>

<div class="flex flex-col gap-2">
  <label
    class={cn(
      buttonVariants({ variant: 'outline' }),
      importing && 'pointer-events-none opacity-50',
    )}
  >
    <UploadIcon data-icon="inline-start" />
    {label}データ（CSV）を{importing ? '取り込み中…' : '選択'}
    <input
      type="file"
      accept=".csv,text/csv"
      class="hidden"
      disabled={importing}
      onchange={importFile}
    />
  </label>
  {#if error}
    <p class="text-sm whitespace-pre-line text-destructive">{error}</p>
  {/if}
</div>
