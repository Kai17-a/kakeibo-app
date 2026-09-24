<script setup lang="ts">
import { formatCurrency, formatSignedCurrency } from "~/utils/format";

const props = defineProps<{
  monthLabel: string;
  /** Totals including the forecast of recurring items not yet recorded. */
  income: number;
  expense: number;
  /** The recurring amounts included in `income` / `expense`. */
  forecast: { income: number; expense: number };
  /** The sum of this month's budgets; 0 when no budget is set. */
  budgetTotal: number;
  /** Spending against `budgetTotal`, in percent. */
  budgetRate: number;
}>();

const balance = computed(() => props.income - props.expense);
</script>

<template>
  <section
    class="grid grid-cols-2 overflow-hidden rounded-lg border border-default bg-elevated sm:grid-cols-3 sm:divide-x sm:divide-default"
    :aria-label="`${monthLabel}の収支概要`"
  >
    <div class="p-3 sm:p-6">
      <p class="text-sm text-muted">収入</p>
      <p
        :class="[
          'mt-1 text-base font-bold whitespace-nowrap tabular-nums sm:text-2xl',
          income > 0 ? 'text-primary' : 'text-default',
        ]"
      >
        {{ formatSignedCurrency(income, "positive") }}
      </p>
      <UBadge v-if="forecast.income" class="mt-2" color="neutral" variant="soft">
        うち予定 {{ formatCurrency(forecast.income) }}
      </UBadge>
    </div>
    <div class="border-l border-default p-3 sm:border-t-0 sm:p-6">
      <p class="text-sm text-muted">支出</p>
      <p class="mt-1 text-base font-bold whitespace-nowrap tabular-nums sm:text-2xl">
        {{ formatSignedCurrency(expense, "negative") }}
      </p>
      <UBadge v-if="forecast.expense" class="mt-2" color="neutral" variant="soft">
        うち予定 {{ formatCurrency(forecast.expense) }}
      </UBadge>
    </div>
    <div class="col-span-2 border-t border-default p-3 sm:col-span-1 sm:border-t-0 sm:p-6">
      <p class="text-sm text-muted">収支</p>
      <p
        :class="[
          'mt-1 text-lg font-bold whitespace-nowrap tabular-nums sm:text-2xl',
          balance < 0 ? 'text-error' : balance > 0 ? 'text-primary' : 'text-default',
        ]"
      >
        {{ formatSignedCurrency(balance) }}
      </p>
      <UProgress
        v-if="budgetTotal"
        class="mt-2"
        :model-value="Math.min(100, budgetRate)"
        :color="budgetRate > 100 ? 'error' : budgetRate >= 80 ? 'warning' : 'primary'"
      />
      <p v-if="budgetTotal || forecast.expense || forecast.income" class="mt-2 text-xs text-muted">
        {{ budgetTotal ? `予算 使用率 ${Math.round(budgetRate)}%` : "予定を含む" }}
      </p>
    </div>
  </section>
</template>
