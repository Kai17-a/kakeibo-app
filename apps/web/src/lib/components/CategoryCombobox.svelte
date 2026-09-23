<script lang="ts">
  import CheckIcon from '@lucide/svelte/icons/check';
  import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
  import { toast } from 'svelte-sonner';
  import { api } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import * as Command from '$lib/components/ui/command';
  import { Spinner } from '$lib/components/ui/spinner';
  import * as Popover from '$lib/components/ui/popover';
  import type { ExpenseCategory, IncomeCategory } from '$lib/types';

  type Category = ExpenseCategory | IncomeCategory;

  interface Props {
    id: string;
    categories: { id: string; name: string }[];
    value: string;
    kind: 'expense' | 'income';
    oncreate?: (category: Category) => void;
    required?: boolean;
  }

  let { id, categories, value = $bindable(), kind, oncreate, required = false }: Props = $props();
  let open = $state(false);
  let search = $state('');
  let creating = $state(false);

  const selectedCategory = $derived(categories.find((category) => category.id === value));
  const trimmedSearch = $derived(search.trim());
  const canCreate = $derived(
    trimmedSearch.length > 0 && !categories.some((category) => category.name === trimmedSearch),
  );

  function selectCategory(categoryId: string) {
    value = categoryId;
    close();
  }

  async function createCategory() {
    const name = trimmedSearch;
    if (!name || !canCreate || creating) return;

    creating = true;
    try {
      const category =
        kind === 'expense'
          ? await api.createExpenseCategory({ name, description: null, parent_category_id: null })
          : await api.createIncomeCategory({ name, description: null, parent_category_id: null });
      value = category.id;
      oncreate?.(category);
      close();
    } catch (caught) {
      toast.error(caught instanceof Error ? caught.message : 'カテゴリを登録できませんでした。');
    } finally {
      creating = false;
    }
  }

  function close() {
    open = false;
    search = '';
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        {id}
        class="w-full justify-between border border-transparent border-b-input bg-transparent px-0 font-normal tracking-normal normal-case"
        variant="ghost"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-required={required}
        aria-label={selectedCategory ? `カテゴリ: ${selectedCategory.name}` : 'カテゴリを選択'}
        disabled={creating}
      >
        <span class={selectedCategory ? '' : 'text-muted-foreground'}>
          {selectedCategory?.name ?? 'カテゴリを選択'}
        </span>
        <ChevronDownIcon class="opacity-50" aria-hidden="true" />
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-[min(20rem,calc(100vw-2rem))] p-0" align="start">
    <Command.Root label="カテゴリを検索">
      <Command.Input placeholder="カテゴリを検索…" bind:value={search} disabled={creating} />
      <Command.List>
        <Command.Empty>
          {#if !canCreate}カテゴリが見つかりません{/if}
        </Command.Empty>
        <Command.Group>
          {#each categories as category (category.id)}
            <Command.Item
              value={category.name}
              disabled={creating}
              onSelect={() => selectCategory(category.id)}
            >
              {category.name}
              {#if category.id === value}<CheckIcon class="ml-auto" aria-hidden="true" />{/if}
            </Command.Item>
          {/each}
        </Command.Group>
        {#if canCreate}
          <Command.Group>
            <Command.Item
              value={`新規登録 ${trimmedSearch}`}
              disabled={creating}
              onSelect={createCategory}
            >
              {#if creating}<Spinner data-icon="inline-start" />{/if}
              「{trimmedSearch}」を新規登録
            </Command.Item>
          </Command.Group>
        {/if}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
