<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { CategoryInput, ExpenseCategory, IncomeCategory } from '$lib/types';

  type Category = ExpenseCategory | IncomeCategory;

  interface Props {
    kind: 'expense' | 'income';
    saving: boolean;
    categories: Category[];
    initial?: Category;
    onclose(): void;
    onsubmit(input: CategoryInput): Promise<void>;
  }

  let { kind, categories, saving, initial, onclose, onsubmit }: Props = $props();
  let name = $state(untrack(() => initial?.name ?? ''));
  let description = $state(untrack(() => initial?.description ?? ''));
  let parentCategoryId = $state(untrack(() => initial?.parent_category_id ?? ''));
  const label = $derived(kind === 'expense' ? '支出カテゴリ' : '収入カテゴリ');
  const editing = $derived(Boolean(initial));
  const editingHasChildren = $derived(
    initial ? categories.some((category) => category.parent_category_id === initial.id) : false,
  );
  const parentCandidates = $derived(
    categories.filter(
      (category) => category.id !== initial?.id && category.parent_category_id === null,
    ),
  );

  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      name: name.trim(),
      description: description.trim() || null,
      parent_category_id: parentCategoryId || null,
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>{label}を{editing ? '編集' : '追加'}</Dialog.Title>
      <Dialog.Description>
        収支の登録時に選択するカテゴリを{editing ? '更新' : '作成'}します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel for="category-name">カテゴリ名</Field.FieldLabel>
          <Input
            id="category-name"
            required
            maxlength={100}
            placeholder={kind === 'expense' ? '例：食費' : '例：給与'}
            bind:value={name}
          />
        </Field.Field>
        <Field.Field data-disabled={editingHasChildren}>
          <Field.FieldLabel for="parent-category">親カテゴリ（任意）</Field.FieldLabel>
          <NativeSelect.Root
            id="parent-category"
            class="w-full"
            disabled={editingHasChildren}
            bind:value={parentCategoryId}
          >
            <NativeSelect.Option value="">親カテゴリなし</NativeSelect.Option>
            {#each parentCandidates as category (category.id)}
              <NativeSelect.Option value={category.id}>{category.name}</NativeSelect.Option>
            {/each}
          </NativeSelect.Root>
          {#if editingHasChildren}
            <Field.FieldDescription>
              子カテゴリが存在するため、このカテゴリには親を設定できません。
            </Field.FieldDescription>
          {/if}
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="category-description">説明（任意）</Field.FieldLabel>
          <Textarea
            id="category-description"
            maxlength={500}
            placeholder="カテゴリの用途や対象を入力"
            bind:value={description}
          />
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving || !name.trim()}>
          {#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}{label}を{editing ? '更新' : '追加'}{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
