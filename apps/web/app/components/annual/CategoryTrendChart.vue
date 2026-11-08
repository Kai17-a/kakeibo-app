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
      <div class="flex h-72 items-end gap-1 border-b border-default sm:gap-2">
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
          <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs"
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
    </template>
    <p v-else class="py-24 text-center text-sm text-muted">この年の支出はありません。</p>
  </UCard>
</template>
