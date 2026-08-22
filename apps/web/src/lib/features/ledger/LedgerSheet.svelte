<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Tabs from '$lib/components/ui/tabs';
  import { categoryTotals, sumAmounts } from '../../domain/summaries';
  import { formatDate, formatYen } from '../../format';
  import { cn } from '../../utils';
  import type {
    Expense,
    ExpenseCategory,
    Income,
    IncomeCategory,
    PaymentMethod,
    RecurringExpense,
  } from '../../types';
  import DailyCategoryTable from './DailyCategoryTable.svelte';
  interface Props {
    month: string;
    monthLabel: string;
    expenses: Expense[];
    incomes: Income[];
    expenseCategories: ExpenseCategory[];
    incomeCategories: IncomeCategory[];
    paymentMethods: PaymentMethod[];
    recurringExpenses: RecurringExpense[];
    onedit(item: Expense): void;
    ondelete(item: Expense): void;
  }
  let {
    month,
    monthLabel,
    expenses,
    incomes,
    expenseCategories,
    incomeCategories,
    paymentMethods,
    recurringExpenses,
    onedit,
    ondelete,
  }: Props = $props();
  let tab = $state<'summary' | 'details' | 'categories'>('summary');
  const incomeTotal = $derived(sumAmounts(incomes));
  const expenseTotal = $derived(sumAmounts(expenses));
  const paymentNames = $derived(new SvelteMap(paymentMethods.map((item) => [item.id, item.name])));
  const categoryNames = $derived(
    new SvelteMap(expenseCategories.map((item) => [item.id, item.name])),
  );
  const incomeBreakdown = $derived(
    incomeCategories.map((category) => ({
      ...category,
      total: sumAmounts(incomes.filter((item) => item.category_id === category.id)),
    })),
  );
  const paymentBreakdown = $derived(
    paymentMethods.map((method) => ({
      ...method,
      total: sumAmounts(expenses.filter((item) => item.payment_method_id === method.id)),
    })),
  );
  const recurring = $derived(
    recurringExpenses.filter(
      (item) =>
        item.is_active &&
        item.start_date.slice(0, 7) <= month &&
        (!item.end_date || item.end_date.slice(0, 7) >= month),
    ),
  );
  const variable = $derived(expenses.filter((item) => !item.recurring_expense_id));
  const recurringTotal = $derived(sumAmounts(recurring));
  const variableTotal = $derived(sumAmounts(variable));
  const variableTotals = $derived(categoryTotals(variable, expenseCategories));
  const ledger = $derived(
    [...expenses].sort((a, b) => a.transaction_date.localeCompare(b.transaction_date)),
  );
  function headClass(...extra: string[]) {
    return cn(
      'px-3 py-2 text-xs font-medium tracking-wider text-muted-foreground uppercase',
      extra,
    );
  }
</script>

<Card.Root class="overflow-hidden">
  <Card.Header>
    <Card.Title>{monthLabel} 家計簿</Card.Title>
    <Card.Description>単位：円</Card.Description>
  </Card.Header>
  <Card.Content class="px-0">
    <Tabs.Root bind:value={tab}>
      <Tabs.List class="mx-4" aria-label="家計簿シート">
        <Tabs.Trigger value="summary">収支・明細</Tabs.Trigger>
        <Tabs.Trigger value="details">支出明細</Tabs.Trigger>
        <Tabs.Trigger value="categories">月ごとのカテゴリ別支出</Tabs.Trigger>
      </Tabs.List>

      {#if tab === 'categories'}
        <DailyCategoryTable {month} {expenses} categories={expenseCategories} />
      {:else if tab === 'details'}
        <section class="min-w-0 p-4">
          <h3 class="bg-muted px-3 py-2 text-sm font-bold">支出明細</h3>
          <div class="max-h-[38rem] overflow-auto">
            <table class="w-full min-w-[800px] text-sm">
              <thead class="sticky top-0 bg-background"
                ><tr class="border-b"
                  ><th class={headClass('w-32 text-left whitespace-nowrap')}>日付</th><th
                    class={headClass('min-w-28 text-left')}>摘要</th
                  ><th class={headClass('min-w-28 text-left')}>支払種別</th><th
                    class={headClass('w-32 text-right whitespace-nowrap')}>金額</th
                  ><th class={headClass('min-w-40 text-left')}>備考</th><th
                    class="w-36 px-3 py-2 text-right"><span class="sr-only">操作</span></th
                  ></tr
                ></thead
              ><tbody
                >{#each ledger as item (item.id)}<tr
                    class="border-t transition-colors hover:bg-muted/50"
                    ><td class="px-3 py-2 whitespace-nowrap">{formatDate(item.transaction_date)}</td
                    ><td class="px-3 py-2">{categoryNames.get(item.category_id)}</td><td
                      class="px-3 py-2">{paymentNames.get(item.payment_method_id)}</td
                    ><td class="px-3 py-2 text-right whitespace-nowrap"
                      >{Number(item.amount).toLocaleString('ja-JP')}</td
                    ><td class="px-3 py-2">{item.description ?? ''}</td><td class="px-3 py-2"
                      ><div class="flex justify-end gap-2">
                        <Button
                          variant="ghost"
                          size="sm"
                          aria-label={`${formatDate(item.transaction_date)} ${categoryNames.get(item.category_id) ?? ''}を編集`}
                          onclick={() => onedit(item)}>編集</Button
                        ><Button
                          variant="destructive"
                          size="sm"
                          aria-label={`${formatDate(item.transaction_date)} ${categoryNames.get(item.category_id) ?? ''}を削除`}
                          onclick={() => ondelete(item)}>削除</Button
                        >
                      </div></td
                    ></tr
                  >{/each}</tbody
              >
            </table>
            {#if !ledger.length}<p class="py-12 text-center text-sm text-muted-foreground">
                この月の支出明細はありません。
              </p>{/if}
          </div>
        </section>
      {:else}
        <div class="grid xl:grid-cols-[18rem_1fr]">
          <aside class="border-r p-4">
            <h3 class="bg-muted px-3 py-2 text-sm font-bold">収支サマリー</h3>
            <dl class="grid grid-cols-2 text-sm">
              <dt class="p-2">収入</dt>
              <dd class="p-2 text-right">{formatYen(incomeTotal)}</dd>
              <dt class="p-2">支出</dt>
              <dd class="p-2 text-right">{formatYen(expenseTotal)}</dd>
              <dt class="p-2 font-bold">収支</dt>
              <dd class="p-2 text-right font-bold">
                {formatYen(incomeTotal - expenseTotal)}
              </dd>
            </dl>
            <h3 class="mt-4 bg-muted px-3 py-2 text-sm font-bold">収入</h3>
            {#each incomeBreakdown as item (item.id)}<div class="flex justify-between p-2 text-sm">
                <span>{item.name}</span><span>{item.total.toLocaleString('ja-JP')}</span>
              </div>{/each}
            <h3 class="mt-4 bg-muted px-3 py-2 text-sm font-bold">支払種別</h3>
            {#each paymentBreakdown as item (item.id)}<div class="flex justify-between p-2 text-sm">
                <span>{item.name}</span><span>{item.total.toLocaleString('ja-JP')}</span>
              </div>{/each}
          </aside>
          <div class="p-4">
            <div class="flex items-center justify-between gap-4 bg-muted px-3 py-2">
              <h3 class="text-sm font-bold">支出（固定費）</h3>
              <span class="text-sm font-bold whitespace-nowrap">{formatYen(recurringTotal)}</span>
            </div>
            {#each recurring as item (item.id)}<div
                class="flex items-center justify-between border-b p-2 text-sm"
              >
                <span>{item.name}<small class="block">毎月{item.payment_day}日</small></span><span
                  >{Number(item.amount).toLocaleString('ja-JP')}</span
                >
              </div>{/each}
            <div class="mt-4 flex items-center justify-between gap-4 bg-muted px-3 py-2">
              <h3 class="text-sm font-bold">支出（変動費）</h3>
              <span class="text-sm font-bold whitespace-nowrap">{formatYen(variableTotal)}</span>
            </div>
            {#each variableTotals as item (item.id)}<div
                class="flex justify-between border-b p-2 text-sm"
              >
                <span>{item.name}</span><span>{item.total.toLocaleString('ja-JP')}</span>
              </div>{/each}
          </div>
        </div>
      {/if}
    </Tabs.Root>
  </Card.Content>
</Card.Root>
