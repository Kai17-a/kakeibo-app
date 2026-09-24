<script setup lang="ts">
defineProps<{
  /** The years offered in the select, e.g. the years that have data. */
  years: string[];
}>();

/** The selected year (YYYY), changed with the previous/next buttons or the select. */
const year = defineModel<string>({ required: true });
</script>

<template>
  <UFieldGroup>
    <UButton
      icon="i-lucide-chevron-left"
      color="neutral"
      variant="outline"
      aria-label="前年"
      @click="year = String(Number(year) - 1)"
    />
    <USelect
      :model-value="year"
      :items="years.map((item) => ({ label: `${item}年`, value: item }))"
      aria-label="対象年"
      class="w-28"
      @update:model-value="(value) => (year = String(value))"
    />
    <UButton
      icon="i-lucide-chevron-right"
      color="neutral"
      variant="outline"
      aria-label="翌年"
      @click="year = String(Number(year) + 1)"
    />
  </UFieldGroup>
</template>
