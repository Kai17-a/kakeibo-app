<script setup lang="ts">
import type { ProjectedMonth } from "~/utils/annual";
import { formatCurrency } from "~/utils/format";

const props = defineProps<{
  months: ProjectedMonth[];
}>();

/** The largest monthly income or expense; bars are scaled against it. */
const max = computed(() =>
  Math.max(1, ...props.months.flatMap((item) => [item.income, item.expense])),
);
</script>

<template>
  <UCard class="min-w-0">
    <template #header>
      <h2 class="text-lg font-semibold">月別の収支推移</h2>
    </template>
    <div class="flex h-72 items-end gap-1 border-b border-default sm:gap-2">
      <div
        v-for="item in months"
        :key="item.month"
        class="flex h-full min-w-0 flex-1 flex-col justify-end"
      >
        <div class="flex h-[calc(100%_-_2rem)] items-end justify-center gap-0.5 sm:gap-1">
          <div
            class="w-1.5 bg-primary sm:w-3"
            :style="{ height: `${(item.income / max) * 100}%` }"
            :title="formatCurrency(item.income)"
          />
          <div
            class="w-1.5 bg-error sm:w-3"
            :style="{ height: `${(item.expense / max) * 100}%` }"
            :title="formatCurrency(item.expense)"
          />
        </div>
        <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs"
          >{{ item.month }}月</span
        >
      </div>
    </div>
    <div class="mt-4 flex gap-4 text-xs text-muted">
      <span class="flex items-center gap-1.5"><span class="size-2.5 bg-primary" />収入</span>
      <span class="flex items-center gap-1.5"><span class="size-2.5 bg-error" />支出</span>
    </div>
  </UCard>
</template>
