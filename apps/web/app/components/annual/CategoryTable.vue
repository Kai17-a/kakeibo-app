<script setup lang="ts">
import type { annualCategoryRows } from "~/utils/annual";
import { formatCurrency } from "~/utils/format";

defineProps<{
  /** One row per category, with 12 monthly values. */
  rows: ReturnType<typeof annualCategoryRows>;
}>();

const monthNumbers = Array.from({ length: 12 }, (_, index) => index + 1);
</script>

<template>
  <UCard>
    <template #header>
      <h2 class="text-lg font-semibold">カテゴリ別年間集計</h2>
      <p class="text-sm text-muted">未計上の固定定期支出を予測額として含みます。</p>
    </template>
    <div class="overflow-x-auto">
      <table class="w-full min-w-[900px] text-sm">
        <thead>
          <tr class="border-b border-default">
            <th
              class="px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
            >
              カテゴリ
            </th>
            <th
              v-for="month in monthNumbers"
              :key="month"
              class="px-3 py-2 text-right text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
            >
              {{ month }}月
            </th>
            <th
              class="px-3 py-2 text-right text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
            >
              年間合計
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="row.id" class="border-t border-default">
            <th class="px-3 py-2 text-left whitespace-nowrap">
              {{ row.name }}
            </th>
            <td
              v-for="(value, index) in row.values"
              :key="index"
              class="px-3 py-2 text-right whitespace-nowrap tabular-nums"
            >
              {{ formatCurrency(value) }}
            </td>
            <td class="px-3 py-2 text-right font-bold whitespace-nowrap tabular-nums">
              {{ formatCurrency(row.total) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </UCard>
</template>
