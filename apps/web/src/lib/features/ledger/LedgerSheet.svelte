<script lang="ts">
  import { SvelteMap } from "svelte/reactivity";
  import { categoryTotals, sumAmounts } from "../../domain/summaries";
  import { formatDate, formatYen } from "../../format";
  import type {
    Expense,
    ExpenseCategory,
    Income,
    IncomeCategory,
    PaymentMethod,
    RecurringExpense,
  } from "../../types";
  import DailyCategoryTable from "./DailyCategoryTable.svelte";
  interface Props {
    month: string;
    monthLabel: string;
    expenses: Expense[];
    incomes: Income[];
    expenseCategories: ExpenseCategory[];
    incomeCategories: IncomeCategory[];
    paymentMethods: PaymentMethod[];
    recurringExpenses: RecurringExpense[];
    onRecurring(): void;
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
    onRecurring,
  }: Props = $props();
  let tab = $state<"summary" | "details" | "categories">("summary");
  const incomeTotal = $derived(sumAmounts(incomes));
  const expenseTotal = $derived(sumAmounts(expenses));
  const paymentNames = $derived(
    new SvelteMap(paymentMethods.map((item) => [item.id, item.name])),
  );
  const categoryNames = $derived(
    new SvelteMap(expenseCategories.map((item) => [item.id, item.name])),
  );
  const incomeBreakdown = $derived(
    incomeCategories.map((category) => ({
      ...category,
      total: sumAmounts(
        incomes.filter((item) => item.category_id === category.id),
      ),
    })),
  );
  const paymentBreakdown = $derived(
    paymentMethods.map((method) => ({
      ...method,
      total: sumAmounts(
        expenses.filter((item) => item.payment_method_id === method.id),
      ),
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
  const variable = $derived(
    expenses.filter((item) => !item.recurring_expense_id),
  );
  const variableTotals = $derived(categoryTotals(variable, expenseCategories));
  const ledger = $derived(
    [...expenses].sort((a, b) =>
      a.transaction_date.localeCompare(b.transaction_date),
    ),
  );
</script>

<div class="overflow-hidden rounded-2xl border bg-white">
  <div class="flex justify-between border-b bg-[#eef3ed] px-5 py-3">
    <div><h2 class="font-serif text-xl">{monthLabel} 家計簿</h2></div>
    <span class="text-xs font-bold">単位：円</span>
  </div>
  <nav
    class="flex gap-1 overflow-x-auto border-b bg-[#faf9f5] px-4 pt-3"
    aria-label="家計簿シート"
  >
    <button
      class={[
        "whitespace-nowrap rounded-t-lg px-5 py-2.5 text-sm font-bold",
        tab === "summary" ? "border bg-white text-[#245c4a]" : "text-[#707a74]",
      ]}
      onclick={() => (tab = "summary")}>収支・明細</button
    >
    <button
      class={[
        "whitespace-nowrap rounded-t-lg px-5 py-2.5 text-sm font-bold",
        tab === "details" ? "border bg-white text-[#245c4a]" : "text-[#707a74]",
      ]}
      onclick={() => (tab = "details")}>支出明細</button
    >
    <button
      class={[
        "whitespace-nowrap rounded-t-lg px-5 py-2.5 text-sm font-bold",
        tab === "categories"
          ? "border bg-white text-[#245c4a]"
          : "text-[#707a74]",
      ]}
      onclick={() => (tab = "categories")}>月ごとのカテゴリ別支出</button
    >
  </nav>

  {#if tab === "categories"}
    <DailyCategoryTable {month} {expenses} categories={expenseCategories} />
  {:else if tab === "details"}
    <section class="min-w-0 p-4">
      <h3 class="bg-[#e8ece7] px-3 py-2 text-sm font-bold">支出明細</h3>
      <div class="max-h-[38rem] overflow-auto">
        <table class="w-full min-w-[600px] text-sm">
          <thead class="sticky top-0 bg-white"
            ><tr
              ><th class="p-2 text-left">日付</th><th class="text-left">摘要</th
              ><th class="text-left">支払種別</th><th class="text-right"
                >金額</th
              ><th class="text-left">備考</th></tr
            ></thead
          ><tbody
            >{#each ledger as item (item.id)}<tr class="border-t"
                ><td class="p-2">{formatDate(item.transaction_date)}</td><td
                  >{categoryNames.get(item.category_id)}</td
                ><td>{paymentNames.get(item.payment_method_id)}</td><td
                  class="text-right"
                  >{Number(item.amount).toLocaleString("ja-JP")}</td
                ><td class="pl-3">{item.description ?? ""}</td></tr
              >{/each}</tbody
          >
        </table>
        {#if !ledger.length}<p class="py-12 text-center text-sm text-[#8a918d]">
            この月の支出明細はありません。
          </p>{/if}
      </div>
    </section>
  {:else}
    <div class="grid xl:grid-cols-[18rem_1fr]">
      <aside class="border-r p-4">
        <h3 class="bg-[#dce9df] px-3 py-2 text-sm font-bold">収支サマリー</h3>
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
        <h3 class="mt-4 bg-[#e8ece7] px-3 py-2 text-sm font-bold">収入</h3>
        {#each incomeBreakdown as item (item.id)}<div
            class="flex justify-between p-2 text-sm"
          >
            <span>{item.name}</span><span
              >{item.total.toLocaleString("ja-JP")}</span
            >
          </div>{/each}
        <h3 class="mt-4 bg-[#e8ece7] px-3 py-2 text-sm font-bold">支払種別</h3>
        {#each paymentBreakdown as item (item.id)}<div
            class="flex justify-between p-2 text-sm"
          >
            <span>{item.name}</span><span
              >{item.total.toLocaleString("ja-JP")}</span
            >
          </div>{/each}
      </aside>
      <div class="p-4">
        <div class="flex justify-between bg-[#e6eadf] px-3 py-2">
          <h3 class="text-sm font-bold">支出（固定費）</h3>
          <button class="text-xs font-bold text-[#245c4a]" onclick={onRecurring}
            >＋ 登録</button
          >
        </div>
        {#each recurring as item (item.id)}<div
            class="flex justify-between border-b p-2 text-sm"
          >
            <span
              >{item.name}<small class="block">毎月{item.payment_day}日</small
              ></span
            ><span>{Number(item.amount).toLocaleString("ja-JP")}</span>
          </div>{/each}
        <div class="flex justify-between p-2 font-bold">
          <span>合計</span><span
            >{sumAmounts(recurring).toLocaleString("ja-JP")}</span
          >
        </div>
        <h3 class="mt-4 bg-[#f0e8dc] px-3 py-2 text-sm font-bold">
          支出（変動費）
        </h3>
        {#each variableTotals as item (item.id)}<div
            class="flex justify-between border-b p-2 text-sm"
          >
            <span>{item.name}</span><span
              >{item.total.toLocaleString("ja-JP")}</span
            >
          </div>{/each}
        <div
          class="flex justify-between border-t-2 border-[#d9d2c7] p-2 font-bold"
        >
          <span>合計</span><span
            >{sumAmounts(variable).toLocaleString("ja-JP")}</span
          >
        </div>
      </div>
    </div>
  {/if}
</div>
