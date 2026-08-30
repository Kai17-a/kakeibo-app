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
    Income,
    IncomeCategory,
    IncomeInput,
    PaymentMethod,
    RecurringExpense,
    RecurringIncome,
  } from '../../types';
  interface Props {
    expenseCategories: ExpenseCategory[];
    incomeCategories: IncomeCategory[];
    paymentMethods: PaymentMethod[];
    saving: boolean;
    initialDate: string;
    initialExpense?: Expense;
    initialIncome?: Income;
    initialRecurring?: RecurringExpense;
    initialRecurringIncome?: RecurringIncome;
    onclose(): void;
    onsubmit(
      kind: 'expense' | 'income',
      input: ExpenseInput | IncomeInput,
      keepOpen: boolean,
    ): Promise<boolean>;
  }
  let {
    expenseCategories,
    incomeCategories,
    paymentMethods,
    saving,
    initialDate,
    initialExpense,
    initialIncome,
    initialRecurring,
    initialRecurringIncome,
    onclose,
    onsubmit,
  }: Props = $props();
  let kind = $derived<'expense' | 'income'>(
    initialIncome || initialRecurringIncome ? 'income' : 'expense',
  );
  const categories = $derived(kind === 'expense' ? expenseCategories : incomeCategories);
  const editing = $derived(Boolean(initialExpense ?? initialIncome));
  const preset = $derived(Boolean(initialRecurring ?? initialRecurringIncome));
  const recurringDate = $derived.by(() => {
    const recurring = initialRecurring ?? initialRecurringIncome;
    if (!recurring) return undefined;
    const [yearValue, monthValue] = initialDate.split('-').map(Number);
    const year = yearValue;
    const monthIndex = monthValue - 1;
    const lastDay = new Date(year, monthIndex + 1, 0).getDate();
    const day = Math.min(recurring.payment_day, lastDay);
    return `${year}-${String(monthIndex + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
  });
  let date = $derived(
    initialExpense?.transaction_date ??
      initialIncome?.transaction_date ??
      recurringDate ??
      initialDate,
  );
  let amount = $derived(
    initialExpense?.amount ??
      initialIncome?.amount ??
      initialRecurring?.amount ??
      initialRecurringIncome?.amount ??
      '',
  );
  let categoryId = $derived(
    initialExpense?.category_id ??
      initialIncome?.category_id ??
      initialRecurring?.category_id ??
      initialRecurringIncome?.category_id ??
      categories[0]?.id ??
      '',
  );
  let paymentMethodId = $derived(
    initialExpense?.payment_method_id ??
      initialRecurring?.payment_method_id ??
      paymentMethods[0]?.id ??
      '',
  );
  let description = $derived(
    initialExpense?.description ??
      initialIncome?.description ??
      (initialRecurring ?? initialRecurringIncome)?.name ??
      '',
  );
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    const keepOpen = (event.submitter as HTMLButtonElement | null)?.value === 'continue';
    const saved = await onsubmit(
      kind,
      kind === 'expense'
        ? {
            transaction_date: date,
            amount: String(amount),
            category_id: categoryId,
            payment_method_id: paymentMethodId,
            recurring_expense_id:
              initialExpense?.recurring_expense_id ?? initialRecurring?.id ?? null,
            description: description || null,
          }
        : {
            transaction_date: date,
            amount: String(amount),
            category_id: categoryId,
            recurring_income_id:
              initialIncome?.recurring_income_id ?? initialRecurringIncome?.id ?? null,
            description: description || null,
          },
      keepOpen,
    );
    if (saved && keepOpen) {
      amount = '';
      description = '';
      document.getElementById('transaction-amount')?.focus();
    }
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>
        {initialExpense
          ? '支出を編集'
          : initialIncome
            ? '収入を編集'
            : preset
              ? initialRecurringIncome
                ? '準固定収入を登録'
                : '準固定費を登録'
              : '収支を登録'}
      </Dialog.Title>
      <Dialog.Description>日付や金額、カテゴリを入力してください。</Dialog.Description>
    </Dialog.Header>
    {#if !editing && !preset}
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
        {#if !editing && !preset}
          <Button variant="outline" type="submit" name="intent" value="continue" disabled={saving}>
            {#if saving}<Spinner data-icon="inline-start" />保存中…{:else}登録して続ける{/if}
          </Button>
        {/if}
        <Button type="submit" disabled={saving}>
          {#if saving}<Spinner
              data-icon="inline-start"
            />保存中…{:else if editing}更新する{:else}登録する{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
