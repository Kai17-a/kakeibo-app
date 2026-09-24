<script setup lang="ts">
import type { paymentMethodBalanceTrend } from "~/utils/summaries";
import { divergingScale } from "~/utils/annual";
import { seriesColor } from "~/utils/chart-colors";
import { formatCurrency } from "~/utils/format";

const props = defineProps<{
  /** Month-end balance per payment method, as returned by paymentMethodBalanceTrend. */
  months: ReturnType<typeof paymentMethodBalanceTrend>;
}>();

/** Every month lists the same payment methods, so the first month describes the legend. */
const legend = computed(() => props.months[0]?.values ?? []);
const scale = computed(() =>
  divergingScale(props.months.flatMap((item) => item.values.map((value) => value.balance))),
);

function barStyle(balance: number, index: number, count: number) {
  const { range, zeroOffset } = scale.value;
  return {
    backgroundColor: seriesColor(index),
    left: `${(index / count) * 100}%`,
    width: `${100 / count}%`,
    top: balance >= 0 ? `${zeroOffset - (balance / range) * 100}%` : `${zeroOffset}%`,
    height: `${(Math.abs(balance) / range) * 100}%`,
  };
}
</script>

<template>
  <UCard class="min-w-0">
    <template #header>
      <h2 class="text-lg font-semibold">資産残高推移</h2>
      <p class="text-sm text-muted">支払方法ごとの月末残高</p>
    </template>
    <template v-if="legend.length">
      <div class="relative flex h-72 gap-1 sm:gap-2">
        <div v-for="item in months" :key="item.month" class="flex h-full min-w-0 flex-1 flex-col">
          <div class="relative h-[calc(100%_-_2rem)]">
            <div
              class="pointer-events-none absolute inset-x-0 border-t border-default"
              :style="{ top: `${scale.zeroOffset}%` }"
            />
            <div
              v-for="(value, index) in item.values"
              :key="value.id"
              class="absolute"
              :style="barStyle(value.balance, index, item.values.length)"
              :title="`${value.name}: ${formatCurrency(value.balance)}`"
            />
          </div>
          <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs"
            >{{ item.month }}月</span
          >
        </div>
      </div>
      <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted">
        <span v-for="(method, index) in legend" :key="method.id" class="flex items-center gap-1.5">
          <span class="size-2.5" :style="{ backgroundColor: seriesColor(index) }" />
          {{ method.name }}
        </span>
      </div>
    </template>
    <p v-else class="py-24 text-center text-sm text-muted">
      初期残高が設定された支払方法はありません。
    </p>
  </UCard>
</template>
