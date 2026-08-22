<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import * as Table from '$lib/components/ui/table';
  import {
    annualMonthlyTotals,
    categoryTotals,
    inPeriod,
    sumAmounts,
  } from '../../domain/summaries';
  import { formatYen } from '../../format';
  import type { Expense, ExpenseCategory, Income } from '../../types';
  interface Props {
    year: string;
    expenses: Expense[];
    incomes: Income[];
    categories: ExpenseCategory[];
  }
  let { year, expenses, incomes, categories }: Props = $props();
  const annualExpenses = $derived(inPeriod(expenses, year));
  const annualIncomes = $derived(inPeriod(incomes, year));
  const expenseTotal = $derived(sumAmounts(annualExpenses));
  const incomeTotal = $derived(sumAmounts(annualIncomes));
  const balance = $derived(incomeTotal - expenseTotal);
  const months = $derived(annualMonthlyTotals(expenses, incomes, year));
  const chartMax = $derived(Math.max(1, ...months.flatMap((item) => [item.income, item.expense])));
  const spending = $derived(
    categoryTotals(annualExpenses, categories)
      .filter((item) => item.total)
      .sort((a, b) => b.total - a.total),
  );
</script>

<section class="grid gap-4 md:grid-cols-3">
  <Card.Root>
    <Card.Header>
      <Card.Description>年間収入</Card.Description><Card.Title>{formatYen(incomeTotal)}</Card.Title>
    </Card.Header><Card.Content>月平均 {formatYen(incomeTotal / 12)}</Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header>
      <Card.Description>年間支出</Card.Description><Card.Title>
        {formatYen(expenseTotal)}
      </Card.Title>
    </Card.Header><Card.Content>月平均 {formatYen(expenseTotal / 12)}</Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header>
      <Card.Description>年間収支</Card.Description><Card.Title>{formatYen(balance)}</Card.Title>
    </Card.Header><Card.Content>
      貯蓄率 {incomeTotal ? Math.round((balance / incomeTotal) * 100) : 0}%
    </Card.Content>
  </Card.Root>
</section>
<div class="mt-8 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
  <Card.Root>
    <Card.Header><Card.Title>月別の収支推移</Card.Title></Card.Header>
    <Card.Content>
      <div class="flex h-72 items-end gap-2 border-b">
        {#each months as item (item.month)}<div class="flex h-full flex-1 flex-col justify-end">
            <div class="flex h-[calc(100%_-_2rem)] items-end justify-center gap-1">
              <div
                class="w-3 bg-chart-1"
                style:height={`${(item.income / chartMax) * 100}%`}
                title={formatYen(item.income)}
              ></div>
              <div
                class="w-3 bg-chart-3"
                style:height={`${(item.expense / chartMax) * 100}%`}
                title={formatYen(item.expense)}
              ></div>
            </div>
            <span class="mt-2 text-center text-xs">{item.month}月</span>
          </div>{/each}
      </div>
    </Card.Content>
  </Card.Root>
  <Card.Root>
    <Card.Header><Card.Title>年間支出の内訳</Card.Title></Card.Header>
    <Card.Content class="grid gap-4">
      {#each spending.slice(0, 6) as item (item.id)}<div class="flex justify-between text-sm">
          <b>{item.name}</b>
          <span>{formatYen(item.total)}</span>
        </div>{/each}
    </Card.Content>
  </Card.Root>
</div>
<Card.Root class="mt-6 overflow-hidden">
  <Card.Header><Card.Title>月ごとの収支</Card.Title></Card.Header>
  <Card.Content class="px-0">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head>月</Table.Head><Table.Head class="text-right">収入</Table.Head><Table.Head
            class="text-right"
          >
            支出
          </Table.Head><Table.Head class="text-right">収支</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each months as item (item.month)}<Table.Row>
            <Table.Head>{item.month}月</Table.Head><Table.Cell class="text-right">
              {formatYen(item.income)}
            </Table.Cell><Table.Cell class="text-right">
              {formatYen(item.expense)}
            </Table.Cell><Table.Cell class="text-right font-bold">
              {formatYen(item.balance)}
            </Table.Cell>
          </Table.Row>{/each}
      </Table.Body>
    </Table.Root>
  </Card.Content>
</Card.Root>
