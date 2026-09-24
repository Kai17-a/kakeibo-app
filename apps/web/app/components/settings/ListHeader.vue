<script setup lang="ts">
defineProps<{
  headingId: string;
  title: string;
  description: string;
  /** Shown as a badge next to the title; hidden while null (e.g. loading or failed). */
  count: number | null;
  addDisabled?: boolean;
}>();

const emit = defineEmits<{ add: [] }>();
</script>

<template>
  <div class="flex min-w-0 items-start justify-between gap-3 sm:gap-4">
    <div class="min-w-0 space-y-1">
      <h2
        :id="headingId"
        class="flex flex-wrap items-center gap-2 text-xl font-semibold text-highlighted"
      >
        {{ title }}
        <UBadge v-if="count !== null" color="neutral" variant="soft"> {{ count }}件 </UBadge>
      </h2>
      <p class="text-sm text-muted">{{ description }}</p>
    </div>
    <UButton icon="i-lucide-plus" class="shrink-0" :disabled="addDisabled" @click="emit('add')">
      追加
    </UButton>
  </div>
</template>
