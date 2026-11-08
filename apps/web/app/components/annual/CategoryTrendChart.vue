<script setup lang="ts">
import type { categoryMonthlyTotals } from "~/utils/summaries";
import { seriesColor } from "~/utils/chart-colors";
import { formatCurrency } from "~/utils/format";

const props = defineProps<{
  /** Per-month spending by category, as returned by categoryMonthlyTotals. */
  months: ReturnType<typeof categoryMonthlyTotals>;
}>();

/** The largest monthly total; stacked bars are scaled against it. */
const max = computed(() => Math.max(1, ...props.months.map((item) => item.total)));
/** Every month lists the same categories, so the first month describes the legend. */
const legend = computed(() => props.months[0]?.values ?? []);
</script>

<template>
  <UCard>
    <template #header>
      <h2 class="text-lg font-semibold">カテゴリ別月次推移</h2>
      <p class="text-sm text-muted">月ごとの支出構成</p>
    </template>
    <template v-if="legend.length">
      <div aria-hidden="true" class="flex h-72 items-end gap-1 border-b border-default sm:gap-2">
        <div
          v-for="item in months"
          :key="item.month"
          class="flex h-full min-w-0 flex-1 flex-col justify-end"
        >
          <div class="flex h-[calc(100%_-_2rem)] flex-col-reverse justify-start">
            <div
              v-for="(value, index) in item.values"
              v-show="value.total"
              :key="value.id"
              class="w-full"
              :style="{
                height: `${(value.total / max) * 100}%`,
                backgroundColor: seriesColor(index),
              }"
              :title="`${value.name}: ${formatCurrency(value.total)}`"
            />
          </div>
          <span class="mt-2 block text-center text-[11px] whitespace-nowrap sm:text-xs"
            >{{ item.month }}月</span
          >
        </div>
      </div>
      <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted">
        <span
          v-for="(category, index) in legend"
          :key="category.id"
          class="flex items-center gap-1.5"
        >
          <span class="size-2.5" :style="{ backgroundColor: seriesColor(index) }" />
          {{ category.name }}
        </span>
      </div>
      <details class="mt-4">
        <summary class="min-h-[44px] cursor-pointer py-3 text-sm text-primary">
          月別の金額を表で見る
        </summary>
        <div class="overflow-x-auto">
          <table class="w-full text-right text-sm tabular-nums">
            <caption class="sr-only">
              カテゴリ別月次推移の金額
            </caption>
            <thead>
              <tr>
                <th scope="col" class="p-2 text-left">月</th>
                <th
                  v-for="category in legend"
                  :key="category.id"
                  scope="col"
                  class="p-2 whitespace-nowrap"
                >
                  {{ category.name }}
                </th>
                <th scope="col" class="p-2">合計</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in months" :key="item.month" class="border-t border-default">
                <th scope="row" class="p-2 text-left whitespace-nowrap">{{ item.month }}月</th>
                <td v-for="value in item.values" :key="value.id" class="p-2">
                  {{ formatCurrency(value.total) }}
                </td>
                <td class="p-2 font-semibold">{{ formatCurrency(item.total) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </details>
    </template>
    <p v-else class="py-24 text-center text-sm text-muted">この年の支出はありません。</p>
  </UCard>
</template>
