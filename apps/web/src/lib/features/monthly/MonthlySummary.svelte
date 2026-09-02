<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import ArrowDownLeftIcon from '@lucide/svelte/icons/arrow-down-left';
  import ArrowUpRightIcon from '@lucide/svelte/icons/arrow-up-right';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Empty from '$lib/components/ui/empty';
  import { Progress } from '$lib/components/ui/progress';
  import {
    budgetActuals,
    categoryTotals,
    mergeTransactions,
    sumAmounts,
  } from '../../domain/summaries';
  import { formatDate, formatYen } from '../../format';
  import type {
    Expense,
    ExpenseCategory,
    Income,
    IncomeCategory,
    PaymentMethod,
    RecurringExpense,
    RecurringIncome,
    Budget,
  } from '../../types';
  interface Props {
    month: string;
    monthLabel: string;
    expenses: Expense[];
    incomes: Income[];
    expenseCategories: ExpenseCategory[];
    incomeCategories: IncomeCategory[];
    paymentMethods: PaymentMethod[];
    recurringExpenses: RecurringExpense[];
    recurringIncomes: RecurringIncome[];
    budgets: Budget[];
    onregistervariable(item: RecurringExpense): void;
    onregistervariableincome(item: RecurringIncome): void;
  }
  let {
    monthLabel,
    month,
    expenses,
    incomes,
    expenseCategories,
    incomeCategories,
    paymentMethods,
    recurringExpenses,
    recurringIncomes,
    budgets,
    onregistervariable,
    onregistervariableincome,
  }: Props = $props();
  const expenseTotal = $derived(sumAmounts(expenses));
  const incomeTotal = $derived(sumAmounts(incomes));
  const balance = $derived(incomeTotal - expenseTotal);
  const transactions = $derived(mergeTransactions(expenses, incomes));
  const fixedRecurring = $derived(
    recurringExpenses.filter((item) => item.is_active && !item.is_variable),
  );
  const variableRecurring = $derived(
    recurringExpenses.filter((item) => item.is_active && item.is_variable),
  );
  const fixedRecurringIncomes = $derived(
    recurringIncomes.filter((item) => item.is_active && !item.is_variable),
  );
  const variableRecurringIncomes = $derived(
    recurringIncomes.filter((item) => item.is_active && item.is_variable),
  );
  const expenseNames = $derived(
    new SvelteMap(expenseCategories.map((item) => [item.id, item.name])),
  );
  const incomeNames = $derived(new SvelteMap(incomeCategories.map((item) => [item.id, item.name])));
  const paymentNames = $derived(new SvelteMap(paymentMethods.map((item) => [item.id, item.name])));
  const spending = $derived(
    categoryTotals(expenses, expenseCategories)
      .filter((item) => item.total)
      .sort((a, b) => b.total - a.total),
  );
  const actuals = $derived(budgetActuals(expenses, expenseCategories, budgets, month));
  function label(item: ReturnType<typeof mergeTransactions>[number]) {
    return (
      item.description ||
      (item.kind === 'expense'
        ? expenseNames.get(item.category_id)
        : incomeNames.get(item.category_id)) ||
      '名称なし'
    );
  }
</script>

<section class="grid gap-4 md:grid-cols-3" aria-label={`${monthLabel}の収支概要`}>
  <Card.Root>
    <Card.Header>
      <Card.Description>収入</Card.Description><Card.Title>{formatYen(incomeTotal)}</Card.Title>
    </Card.Header>
    <Card.Content><Badge variant="secondary">{incomes.length} 件の入金</Badge></Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header>
      <Card.Description>支出</Card.Description><Card.Title>{formatYen(expenseTotal)}</Card.Title>
    </Card.Header>
    <Card.Content><Badge variant="secondary">{expenses.length} 件の支払い</Badge></Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header>
      <Card.Description>残り</Card.Description><Card.Title>{formatYen(balance)}</Card.Title>
    </Card.Header>
    <Card.Content>
      <Progress
        value={incomeTotal ? Math.min(100, Math.max(0, (balance / incomeTotal) * 100)) : 0}
      />
    </Card.Content>
  </Card.Root>
</section>
<div class="mt-8 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
  <Card.Root class="overflow-hidden">
    <Card.Header>
      <Card.Title>最近の明細</Card.Title>
      <Card.Description>{monthLabel}</Card.Description>
      <Card.Action><Badge variant="outline">{transactions.length} 件</Badge></Card.Action>
    </Card.Header>
    <Card.Content class="px-0">
      {#if transactions.length}<ul class="divide-y">
          {#each transactions as item (item.id)}<li class="group flex items-center gap-3 px-5 py-4">
              <span
                class={[
                  'grid size-10 place-items-center rounded-xl',
                  item.kind === 'income'
                    ? 'bg-secondary text-secondary-foreground'
                    : 'bg-muted text-destructive',
                ]}
              >
                {#if item.kind === 'income'}<ArrowDownLeftIcon />{:else}<ArrowUpRightIcon />{/if}
              </span>
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-bold">{label(item)}</p>
                <p class="text-xs text-muted-foreground">
                  {formatDate(item.transaction_date)}{item.kind === 'expense'
                    ? ` · ${paymentNames.get(item.payment_method_id) ?? ''}`
                    : ''}
                </p>
              </div>
              <p class="font-serif font-semibold">
                {item.kind === 'income' ? '+' : '−'}{formatYen(item.amount)}
              </p>
            </li>{/each}
        </ul>{:else}<Empty.Root>
          <Empty.Header>
            <Empty.Title>この月の明細はまだありません</Empty.Title><Empty.Description>
              収入または支出を登録すると、ここに表示されます。
            </Empty.Description>
          </Empty.Header>
        </Empty.Root>{/if}
    </Card.Content>
  </Card.Root>
  <div class="grid content-start gap-6">
    <Card.Root>
      <Card.Header><Card.Title>支出の内訳</Card.Title></Card.Header>
      <Card.Content class="grid gap-4">
        {#each spending as category (category.id)}<div>
            <div class="flex justify-between text-sm">
              <b>{category.name}</b>
              <span>{formatYen(category.total)}</span>
            </div>
            <Progress
              class="mt-1"
              value={expenseTotal ? (category.total / expenseTotal) * 100 : 0}
            />
          </div>{/each}
      </Card.Content>
    </Card.Root>
    <Card.Root>
      <Card.Header>
        <Card.Title>予算実績</Card.Title><Card.Description>{monthLabel}</Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-4">
        {#if actuals.length}
          {#each actuals as item (item.id)}
            <div>
              <div class="flex justify-between gap-3 text-sm">
                <b>{item.name}</b>
                <span>
                  {formatYen(item.actual)} / {formatYen(item.budget)}（{item.achievementRate ===
                  null
                    ? '—'
                    : `${Math.round(item.achievementRate)}%`}）
                </span>
              </div>
              <Progress
                class="mt-1"
                value={item.achievementRate === null ? 0 : Math.min(100, item.achievementRate)}
              />
            </div>
          {/each}
        {:else}<p class="text-sm text-muted-foreground">設定済みの予算はありません。</p>{/if}
      </Card.Content>
    </Card.Root>
    <Card.Root>
      <Card.Header><Card.Title>定期支出</Card.Title></Card.Header>
      <Card.Content>
        <ul class="mt-3 divide-y">
          {#each fixedRecurring as item (item.id)}<li
              class="flex items-center justify-between py-3 text-sm"
            >
              <span>
                <b class="block">{item.name}</b>
                <small>毎月 {item.payment_day} 日</small>
              </span>
              <b>{formatYen(item.amount)}</b>
            </li>{/each}
        </ul>
        {#if variableRecurring.length}
          <h4 class="mt-4 text-xs font-semibold tracking-widest text-muted-foreground uppercase">
            準固定費（金額変動）
          </h4>
          <ul class="mt-2 divide-y">
            {#each variableRecurring as item (item.id)}<li
                class="flex items-center justify-between py-3 text-sm"
              >
                <span>
                  <b class="block">{item.name}</b>
                  <small>毎月 {item.payment_day} 日 · 目安 {formatYen(item.amount)}</small>
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  aria-label={`${item.name}の今月分を登録`}
                  onclick={() => onregistervariable(item)}
                >
                  今月分を登録
                </Button>
              </li>{/each}
          </ul>
        {/if}
      </Card.Content>
    </Card.Root>
    <Card.Root>
      <Card.Header><Card.Title>定期収入</Card.Title></Card.Header>
      <Card.Content>
        <ul class="divide-y">
          {#each fixedRecurringIncomes as item (item.id)}
            <li class="flex items-center justify-between py-3 text-sm">
              <span>
                <b class="block">{item.name}</b>
                <small>毎月 {item.payment_day} 日</small>
              </span>
              <b>{formatYen(item.amount)}</b>
            </li>
          {/each}
        </ul>
        {#if variableRecurringIncomes.length}
          <h4 class="mt-4 text-xs font-semibold tracking-widest text-muted-foreground uppercase">
            準固定収入（金額変動）
          </h4>
          <ul class="mt-2 divide-y">
            {#each variableRecurringIncomes as item (item.id)}
              <li class="flex items-center justify-between py-3 text-sm">
                <span>
                  <b class="block">{item.name}</b>
                  <small>毎月 {item.payment_day} 日 · 目安 {formatYen(item.amount)}</small>
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  aria-label={`${item.name}の今月分を登録`}
                  onclick={() => onregistervariableincome(item)}
                >
                  今月分を登録
                </Button>
              </li>
            {/each}
          </ul>
        {/if}
      </Card.Content>
    </Card.Root>
  </div>
</div>
