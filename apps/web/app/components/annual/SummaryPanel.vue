<script setup lang="ts">
import { formatCurrency, formatSignedCurrency } from "~/utils/format";

const props = defineProps<{
  /** Yearly totals including forecasts of unrecorded recurring items. */
  income: number;
  expense: number;
}>();

const balance = computed(() => props.income - props.expense);
</script>

<template>
  <section
    class="grid grid-cols-2 overflow-hidden rounded-lg border border-default bg-elevated sm:grid-cols-3 sm:divide-x sm:divide-default"
    aria-label="年間収支概要"
  >
    <div class="p-3 sm:p-6">
      <p class="text-sm text-muted">年間収入</p>
      <p
        :class="[
          'mt-1 text-base font-bold whitespace-nowrap tabular-nums sm:text-2xl',
          income > 0 ? 'text-primary' : 'text-default',
        ]"
      >
        {{ formatSignedCurrency(income, "positive") }}
      </p>
      <p class="mt-2 text-sm text-muted tabular-nums">月平均 {{ formatCurrency(income / 12) }}</p>
    </div>
    <div class="border-l border-default p-3 sm:border-t-0 sm:p-6">
      <p class="text-sm text-muted">年間支出</p>
      <p class="mt-1 text-base font-bold whitespace-nowrap tabular-nums sm:text-2xl">
        {{ formatSignedCurrency(expense, "negative") }}
      </p>
      <p class="mt-2 text-sm text-muted tabular-nums">月平均 {{ formatCurrency(expense / 12) }}</p>
    </div>
    <div class="col-span-2 border-t border-default p-3 sm:col-span-1 sm:border-t-0 sm:p-6">
      <p class="text-sm text-muted">年間収支</p>
      <p
        :class="[
          'mt-1 text-base font-bold whitespace-nowrap tabular-nums sm:text-2xl',
          balance < 0 ? 'text-error' : balance > 0 ? 'text-primary' : 'text-default',
        ]"
      >
        {{ formatSignedCurrency(balance) }}
      </p>
      <p class="mt-2 text-sm text-muted tabular-nums">
        貯蓄率 {{ income ? Math.round((balance / income) * 100) : 0 }}%
      </p>
    </div>
  </section>
</template>
