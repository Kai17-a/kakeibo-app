<script setup lang="ts">
import type { budgetActuals } from "~/utils/summaries";
import { formatCurrency } from "~/utils/format";

defineProps<{
  actuals: ReturnType<typeof budgetActuals>;
}>();

type Actual = ReturnType<typeof budgetActuals>[number];

/** Exceeded budgets are errors; budgets at 80% or more are warnings. */
function statusColor(item: Actual) {
  if (item.exceeded) return "error";
  return item.achievementRate !== null && item.achievementRate >= 80 ? "warning" : "primary";
}

const amountClass = { error: "text-error", warning: "text-warning", primary: "text-default" };
</script>

<template>
  <UCard class="bg-default">
    <template #header>
      <h2 class="text-lg font-semibold">予算</h2>
    </template>
    <div class="space-y-4">
      <div v-for="item in actuals" :key="item.id">
        <div class="flex justify-between gap-3 text-sm">
          <b>{{ item.name }}</b>
          <span :class="['tabular-nums', amountClass[statusColor(item)]]"
            >{{ formatCurrency(item.actual) }} / {{ formatCurrency(item.budget) }}（{{
              item.achievementRate === null ? "—" : `${Math.round(item.achievementRate)}%`
            }}）</span
          >
        </div>
        <UProgress
          class="mt-1"
          :model-value="item.achievementRate === null ? 0 : Math.min(100, item.achievementRate)"
          :color="statusColor(item)"
        />
      </div>
      <p v-if="!actuals.length" class="text-sm text-muted">
        予算はまだ設定されていません。
        <NuxtLink to="/settings/budget" class="font-medium text-primary hover:underline"
          >予算を設定</NuxtLink
        >
      </p>
    </div>
  </UCard>
</template>
