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
  <AppAlert
    v-if="error"
    color="error"
    variant="subtle"
    title="データを読み込めませんでした"
    :description="error"
    :actions="[
      { label: '再試行', color: 'error', variant: 'outline', onClick: () => emit('retry') },
    ]"
    class="mb-6 shrink-0"
  />
  <div v-if="loading" class="space-y-4">
    <AppSkeleton class="h-28 w-full" />
    <AppSkeleton class="h-28 w-full" />
  </div>
  <slot v-else />
</template>
