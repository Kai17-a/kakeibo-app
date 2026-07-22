<script lang="ts">
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
  <article class="rounded-2xl border bg-white p-6">
    <p class="text-sm text-[#6d7872]">年間収入</p>
    <p class="mt-3 font-serif text-3xl text-[#245c4a]">{formatYen(incomeTotal)}</p>
    <p class="mt-4 text-xs">月平均 {formatYen(incomeTotal / 12)}</p>
  </article>
  <article class="rounded-2xl border bg-white p-6">
    <p class="text-sm text-[#6d7872]">年間支出</p>
    <p class="mt-3 font-serif text-3xl text-[#b45845]">{formatYen(expenseTotal)}</p>
    <p class="mt-4 text-xs">月平均 {formatYen(expenseTotal / 12)}</p>
  </article>
  <article class="rounded-2xl bg-[#203e34] p-6 text-white">
    <p class="text-sm text-[#bcd3c9]">年間収支</p>
    <p class="mt-3 font-serif text-3xl">{formatYen(balance)}</p>
    <p class="mt-4 text-xs">
      貯蓄率 {incomeTotal ? Math.round((balance / incomeTotal) * 100) : 0}%
    </p>
  </article>
</section>
<div class="mt-8 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
  <section class="rounded-2xl border bg-white p-6">
    <h2 class="font-serif text-xl">月別の収支推移</h2>
    <div class="mt-7 flex h-72 items-end gap-2 border-b">
      {#each months as item (item.month)}<div class="flex h-full flex-1 flex-col justify-end">
          <div class="flex h-[calc(100%_-_2rem)] items-end justify-center gap-1">
            <div
              class="w-3 bg-[#7aac96]"
              style:height={`${(item.income / chartMax) * 100}%`}
              title={formatYen(item.income)}
            ></div>
            <div
              class="w-3 bg-[#d29a62]"
              style:height={`${(item.expense / chartMax) * 100}%`}
              title={formatYen(item.expense)}
            ></div>
          </div>
          <span class="mt-2 text-center text-xs">{item.month}月</span>
        </div>{/each}
    </div>
  </section>
  <section class="rounded-2xl border bg-white p-5">
    <h2 class="font-serif text-xl">年間支出の内訳</h2>
    <div class="mt-5 grid gap-4">
      {#each spending.slice(0, 6) as item (item.id)}<div class="flex justify-between text-sm">
          <b>{item.name}</b><span>{formatYen(item.total)}</span>
        </div>{/each}
    </div>
  </section>
</div>
<section class="mt-6 overflow-hidden rounded-2xl border bg-white">
  <h2 class="border-b px-5 py-4 font-serif text-xl">月ごとの収支</h2>
  <table class="w-full text-sm">
    <thead
      ><tr
        ><th class="px-5 py-3 text-left">月</th><th class="text-right">収入</th><th
          class="text-right">支出</th
        ><th class="px-5 text-right">収支</th></tr
      ></thead
    ><tbody
      >{#each months as item (item.month)}<tr class="border-t"
          ><th class="px-5 py-3 text-left">{item.month}月</th><td class="text-right"
            >{formatYen(item.income)}</td
          ><td class="text-right">{formatYen(item.expense)}</td><td
            class="px-5 text-right font-bold">{formatYen(item.balance)}</td
          ></tr
        >{/each}</tbody
    >
  </table>
</section>
