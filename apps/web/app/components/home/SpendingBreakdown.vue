<script setup lang="ts">
import { formatCurrency } from "~/utils/format";

defineProps<{
  /** Categories with spending, largest first. */
  spending: { id: string; name: string; total: number }[];
  /** This month's total spending, the base of each share. */
  total: number;
}>();
</script>

<template>
  <UCard class="bg-default">
    <template #header>
      <h2 class="text-lg font-semibold">支出の内訳</h2>
    </template>
    <div class="space-y-4">
      <div v-for="category in spending" :key="category.id">
        <div class="flex justify-between text-sm">
          <b>{{ category.name }}</b>
          <span class="tabular-nums"
            >{{ formatCurrency(category.total) }}
            <small class="text-muted"
              >{{ total ? Math.round((category.total / total) * 100) : 0 }}%</small
            ></span
          >
        </div>
        <UProgress class="mt-1" :model-value="total ? (category.total / total) * 100 : 0" />
      </div>
      <p v-if="!spending.length" class="text-sm text-muted">この月の支出はまだありません。</p>
    </div>
  </UCard>
</template>
