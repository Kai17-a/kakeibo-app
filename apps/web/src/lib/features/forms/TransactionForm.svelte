<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import { Spinner } from '$lib/components/ui/spinner';
  import * as Tabs from '$lib/components/ui/tabs';
  import { Textarea } from '$lib/components/ui/textarea';
  import type {
    Expense,
    ExpenseCategory,
    ExpenseInput,
    IncomeCategory,
    IncomeInput,
    PaymentMethod,
  } from '../../types';
  interface Props {
    expenseCategories: ExpenseCategory[];
    incomeCategories: IncomeCategory[];
    paymentMethods: PaymentMethod[];
    saving: boolean;
    initialDate: string;
    initialExpense?: Expense;
    onclose(): void;
    onsubmit(kind: 'expense' | 'income', input: ExpenseInput | IncomeInput): Promise<void>;
  }
  let {
    expenseCategories,
    incomeCategories,
    paymentMethods,
    saving,
    initialDate,
    initialExpense,
    onclose,
    onsubmit,
  }: Props = $props();
  let kind = $state<'expense' | 'income'>('expense');
  const categories = $derived(kind === 'expense' ? expenseCategories : incomeCategories);
  let date = $derived(initialExpense?.transaction_date ?? initialDate);
  let amount = $derived(initialExpense?.amount ?? '');
  let categoryId = $derived(initialExpense?.category_id ?? categories[0]?.id ?? '');
  let paymentMethodId = $derived(initialExpense?.payment_method_id ?? paymentMethods[0]?.id ?? '');
  let description = $derived(initialExpense?.description ?? '');
  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit(
      kind,
      kind === 'expense'
        ? {
            transaction_date: date,
            amount: String(amount),
            category_id: categoryId,
            payment_method_id: paymentMethodId,
            recurring_expense_id: initialExpense?.recurring_expense_id ?? null,
            description: description || null,
          }
        : {
            transaction_date: date,
            amount: String(amount),
            category_id: categoryId,
            description: description || null,
          },
    );
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>{initialExpense ? '支出を編集' : '収支を登録'}</Dialog.Title>
      <Dialog.Description>日付や金額、カテゴリを入力してください。</Dialog.Description>
    </Dialog.Header>
    {#if !initialExpense}
      <Tabs.Root bind:value={kind}>
        <Tabs.List class="grid w-full grid-cols-2">
          <Tabs.Trigger value="expense">支出</Tabs.Trigger>
          <Tabs.Trigger value="income">収入</Tabs.Trigger>
        </Tabs.List>
      </Tabs.Root>
    {/if}
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel for="transaction-date">日付</Field.FieldLabel>
          <Input id="transaction-date" type="date" required bind:value={date} />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="transaction-amount">金額</Field.FieldLabel>
          <Input
            id="transaction-amount"
            type="number"
            min="1"
            step="1"
            required
            bind:value={amount}
          />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="transaction-category">カテゴリ</Field.FieldLabel>
          <NativeSelect.Root id="transaction-category" required bind:value={categoryId}>
            {#each categories as category (category.id)}
              <NativeSelect.Option value={category.id}>{category.name}</NativeSelect.Option>
            {/each}
          </NativeSelect.Root>
        </Field.Field>
        {#if kind === 'expense'}
          <Field.Field>
            <Field.FieldLabel for="transaction-payment">支払方法</Field.FieldLabel>
            <NativeSelect.Root id="transaction-payment" required bind:value={paymentMethodId}>
              {#each paymentMethods as method (method.id)}
                <NativeSelect.Option value={method.id}>{method.name}</NativeSelect.Option>
              {/each}
            </NativeSelect.Root>
          </Field.Field>
        {/if}
        <Field.Field>
          <Field.FieldLabel for="transaction-description">メモ（任意）</Field.FieldLabel>
          <Textarea id="transaction-description" bind:value={description} />
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving}>
          {#if saving}<Spinner
              data-icon="inline-start"
            />保存中…{:else if initialExpense}更新する{:else}登録する{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
