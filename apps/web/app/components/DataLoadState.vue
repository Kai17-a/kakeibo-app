<script setup lang="ts">
defineProps<{
  loading: boolean;
  /** The load error message; shown with a retry action when not empty. */
  error: string;
}>();

const emit = defineEmits<{ retry: [] }>();
</script>

<!-- Renders no wrapper element, so the default slot keeps its place in the parent's layout. -->
<template>
  <UAlert
    v-if="error"
    color="error"
    title="データを読み込めませんでした"
    :description="error"
    :actions="[
      { label: '再試行', color: 'error', variant: 'outline', onClick: () => emit('retry') },
    ]"
    class="mb-6"
  />
  <div v-if="loading" class="space-y-4">
    <USkeleton class="h-28 w-full" />
    <USkeleton class="h-28 w-full" />
  </div>
  <slot v-else />
</template>
