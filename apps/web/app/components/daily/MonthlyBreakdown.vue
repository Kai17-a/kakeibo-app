<script setup lang="ts">
import type { Category, PaymentMethod, RecurringExpense } from "~/types/settings";
import type { Expense, Income } from "~/types/transactions";
import { classifyMonthlyExpenses } from "~/utils/expense-classification";
import { formatCurrency, formatSignedCurrency } from "~/utils/format";
import { categoryTotals, sumAmounts } from "~/utils/summaries";

const props = defineProps<{
  month: string;
  /** The month's recorded transactions. */
  expenses: Expense[];
  incomes: Income[];
  recurringExpenses: RecurringExpense[];
  expenseCategories: Category[];
  incomeCategories: Category[];
  paymentMethods: PaymentMethod[];
}>();

const incomeTotal = computed(() => sumAmounts(props.incomes));
const expenseTotal = computed(() => sumAmounts(props.expenses));
const balance = computed(() => incomeTotal.value - expenseTotal.value);

const incomeBreakdown = computed(() =>
  props.incomeCategories.map((category) => ({
    ...category,
    total: sumAmounts(props.incomes.filter((item) => item.category_id === category.id)),
  })),
);
const paymentBreakdown = computed(() =>
  props.paymentMethods.map((method) => ({
    ...method,
    total: sumAmounts(props.expenses.filter((item) => item.payment_method_id === method.id)),
  })),
);

const classification = computed(() =>
  classifyMonthlyExpenses(props.expenses, props.recurringExpenses, props.month),
);
const nonZeroCategoryTotals = (expenses: Expense[]) =>
  categoryTotals(expenses, props.expenseCategories).filter((item) => item.total);
const semiFixedTotals = computed(() => nonZeroCategoryTotals(classification.value.semiFixed));
const variableTotals = computed(() => nonZeroCategoryTotals(classification.value.variable));
</script>

<template>
  <div class="grid gap-8 lg:grid-cols-2">
    <aside>
      <h2 class="border-b border-default pb-2 text-base font-semibold">収支サマリー</h2>
      <dl class="grid grid-cols-2 text-sm">
        <dt class="p-2">収入</dt>
        <dd
          :class="[
            'p-2 text-right tabular-nums',
            incomeTotal > 0 ? 'text-primary' : 'text-default',
          ]"
        >
          {{ formatSignedCurrency(incomeTotal, "positive") }}
        </dd>
        <dt class="p-2">支出</dt>
        <dd class="p-2 text-right tabular-nums">
          {{ formatSignedCurrency(expenseTotal, "negative") }}
        </dd>
        <dt class="p-2 font-bold">収支</dt>
        <dd
          :class="[
            'p-2 text-right font-bold tabular-nums',
            balance < 0 ? 'text-error' : balance > 0 ? 'text-primary' : 'text-default',
          ]"
        >
          {{ formatSignedCurrency(balance) }}
        </dd>
      </dl>
      <h2 class="mt-5 border-b border-default pb-2 text-base font-semibold">収入</h2>
      <div
        v-for="item in incomeBreakdown"
        :key="item.id"
        class="flex justify-between gap-4 px-2 py-2 text-sm"
      >
        <span>{{ item.name }}</span>
        <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
      </div>
      <h2 class="mt-5 border-b border-default pb-2 text-base font-semibold">支払種別</h2>
      <div
        v-for="item in paymentBreakdown"
        :key="item.id"
        class="flex justify-between gap-4 px-2 py-2 text-sm"
      >
        <span>{{ item.name }}</span>
        <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
      </div>
    </aside>
    <div>
      <div class="flex items-center justify-between gap-4 border-b border-default pb-2">
        <h2 class="text-sm font-bold">支出（固定費）</h2>
        <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
          formatCurrency(sumAmounts(classification.fixed))
        }}</span>
      </div>
      <div
        v-for="item in classification.fixed"
        :key="item.id"
        class="flex items-center justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
      >
        <span>
          {{ item.name }}
          <small class="block text-muted">毎月{{ item.payment_day }}日</small>
        </span>
        <span class="text-right tabular-nums">{{ formatCurrency(item.amount) }}</span>
      </div>
      <div class="mt-5 flex items-center justify-between gap-4 border-b border-default pb-2">
        <h2 class="text-sm font-bold">準固定費（金額変動）</h2>
        <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
          formatCurrency(sumAmounts(classification.semiFixed))
        }}</span>
      </div>
      <div
        v-for="item in semiFixedTotals"
        :key="item.id"
        class="flex justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
      >
        <span>{{ item.name }}</span>
        <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
      </div>
      <div class="mt-5 flex items-center justify-between gap-4 border-b border-default pb-2">
        <h2 class="text-sm font-bold">支出（変動費）</h2>
        <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
          formatCurrency(sumAmounts(classification.variable))
        }}</span>
      </div>
      <div
        v-for="item in variableTotals"
        :key="item.id"
        class="flex justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
      >
        <span>{{ item.name }}</span>
        <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
      </div>
    </div>
  </div>
</template>
