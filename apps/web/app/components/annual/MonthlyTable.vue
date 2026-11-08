<script setup lang="ts">
import type { ProjectedMonth } from "~/utils/annual";
import { formatCurrency, formatSignedCurrency } from "~/utils/format";

defineProps<{
  months: ProjectedMonth[];
}>();

const numeric = { class: { th: "text-right", td: "text-right tabular-nums" } };
const columns = [
  { accessorKey: "month", header: "月" },
  { accessorKey: "income", header: "収入", meta: numeric },
  { accessorKey: "expense", header: "支出", meta: numeric },
  {
    accessorKey: "balance",
    header: "収支",
    meta: { class: { th: "text-right", td: "text-right font-bold tabular-nums" } },
  },
];
</script>

<template>
  <UCard>
    <template #header>
      <h2 class="text-lg font-semibold">月ごとの収支</h2>
    </template>
    <div class="overflow-x-auto">
      <UTable class="min-w-[36rem]" :data="months" :columns="columns">
        <template #month-cell="{ row }"> {{ row.original.month }}月 </template>
        <template #income-cell="{ row }">
          {{ formatCurrency(row.original.income) }}
        </template>
        <template #expense-cell="{ row }">
          {{ formatCurrency(row.original.expense) }}
        </template>
        <template #balance-cell="{ row }">
          <span
            :class="
              row.original.balance < 0
                ? 'text-error'
                : row.original.balance > 0
                  ? 'text-primary'
                  : 'text-default'
            "
          >
            {{ formatSignedCurrency(row.original.balance) }}
          </span>
        </template>
      </UTable>
    </div>
  </UCard>
</template>
