<script setup lang="ts">
import { formatCurrency } from "~/utils/format";

defineProps<{
  /** Categories with spending this year, largest first; the top 8 are shown. */
  spending: { id: string; name: string; total: number }[];
  /** The year's total spending, the base of each share. */
  total: number;
}>();
</script>

<template>
  <UCard class="min-w-0">
    <template #header>
      <h2 class="text-lg font-semibold">年間支出の内訳</h2>
    </template>
    <div class="space-y-3">
      <div v-for="item in spending.slice(0, 8)" :key="item.id" class="text-sm">
        <div class="flex justify-between gap-3">
          <b>{{ item.name }}</b>
          <span class="tabular-nums">
            {{ formatCurrency(item.total) }}
            <small class="text-muted"
              >{{ total ? Math.round((item.total / total) * 100) : 0 }}%</small
            >
          </span>
        </div>
        <UProgress class="mt-1" :model-value="total ? (item.total / total) * 100 : 0" />
      </div>
      <p v-if="!spending.length" class="text-sm text-muted">この年の支出はまだありません。</p>
    </div>
  </UCard>
</template>
