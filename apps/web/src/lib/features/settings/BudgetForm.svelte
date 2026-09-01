<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import { Spinner } from '$lib/components/ui/spinner';
  import type { Budget, BudgetInput, ExpenseCategory } from '$lib/types';
  interface Props {
    saving: boolean;
    categories: ExpenseCategory[];
    initial?: Budget;
    onclose(): void;
    onsubmit(input: BudgetInput): Promise<void>;
  }
  let { saving, categories, initial, onclose, onsubmit }: Props = $props();
  let categoryId = $state(untrack(() => initial?.category_id ?? ''));
  let amount = $state(untrack(() => initial?.amount ?? ''));
  const editing = $derived(Boolean(initial));
  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({ category_id: categoryId, amount: String(amount).trim() });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>予算を{editing ? '編集' : '追加'}</Dialog.Title><Dialog.Description>
        カテゴリごとに毎月適用する予算を設定します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel for="budget-category">支出カテゴリ</Field.FieldLabel>
          <NativeSelect.Root id="budget-category" required bind:value={categoryId}>
            <NativeSelect.Option value="">選択してください</NativeSelect.Option>
            {#each categories as category (category.id)}<NativeSelect.Option value={category.id}>
                {category.name}
              </NativeSelect.Option>{/each}
          </NativeSelect.Root>
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="budget-amount">月額予算</Field.FieldLabel><Input
            id="budget-amount"
            type="number"
            min="0"
            step="1"
            required
            bind:value={amount}
          />
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button><Button
          type="submit"
          disabled={saving || !categoryId || String(amount).trim() === ''}
        >
          {#if saving}<Spinner data-icon="inline-start" />保存中…{:else}保存{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
