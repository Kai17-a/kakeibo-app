<script setup lang="ts">
import { shiftMonth } from "~/utils/format";

/** The selected month (YYYY-MM), changed with the previous/next buttons or the month picker. */
const month = defineModel<string>({ required: true });
</script>

<template>
  <AppFieldGroup class="shrink-0">
    <AppButton
      icon="i-lucide-chevron-left"
      color="neutral"
      variant="outline"
      aria-label="前月"
      @click="month = shiftMonth(month, -1)"
    />
    <MonthPicker v-model="month" class="w-32 sm:w-40" />
    <AppButton
      icon="i-lucide-chevron-right"
      color="neutral"
      variant="outline"
      aria-label="翌月"
      @click="month = shiftMonth(month, 1)"
    />
  </AppFieldGroup>
</template>
