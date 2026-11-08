<script setup lang="ts">
import { shiftMonth } from "~/utils/format";

/** The selected month (YYYY-MM), changed with the previous/next buttons or the month picker. */
const month = defineModel<string>({ required: true });
</script>

<template>
  <UFieldGroup class="shrink-0">
    <UButton
      icon="i-lucide-chevron-left"
      color="neutral"
      variant="outline"
      aria-label="前月"
      @click="month = shiftMonth(month, -1)"
    />
    <MonthPicker v-model="month" class="w-24 sm:w-40" />
    <UButton
      icon="i-lucide-chevron-right"
      color="neutral"
      variant="outline"
      aria-label="翌月"
      @click="month = shiftMonth(month, 1)"
    />
  </UFieldGroup>
</template>
