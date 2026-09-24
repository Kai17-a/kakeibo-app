<script setup lang="ts">
withDefaults(
  defineProps<{
    /** The collection name used in the loading and error messages, e.g. 「支払方法」. */
    label: string;
    loading: boolean;
    error: string;
    empty: boolean;
    emptyIcon: string;
    emptyDescription: string;
    /** The label of the add button shown in the empty state. */
    addLabel: string;
    addDisabled?: boolean;
    /** Height class of each loading placeholder row. */
    skeletonClass?: string;
  }>(),
  { addDisabled: false, skeletonClass: "h-16" },
);

const emit = defineEmits<{ retry: []; add: [] }>();
</script>

<!-- Shows loading, error or empty states, and the default slot once there are items. -->
<template>
  <div v-if="loading" role="status" :aria-label="`${label}を読み込み中`" class="space-y-3">
    <USkeleton v-for="i in 3" :key="i" :class="['w-full', skeletonClass]" />
    <span class="sr-only">読み込み中…</span>
  </div>
  <UAlert
    v-else-if="error"
    color="error"
    variant="subtle"
    :title="`${label}を取得できませんでした`"
    :description="error"
    :actions="[
      { label: '再試行', color: 'error', variant: 'outline', onClick: () => emit('retry') },
    ]"
  />
  <UCard v-else-if="empty" class="text-center">
    <div class="space-y-3 py-10">
      <UIcon :name="emptyIcon" class="size-8 text-muted" />
      <h3 class="font-semibold">{{ label }}がありません</h3>
      <p class="text-sm text-muted">{{ emptyDescription }}</p>
      <UButton
        color="neutral"
        variant="outline"
        icon="i-lucide-plus"
        :disabled="addDisabled"
        @click="emit('add')"
      >
        {{ addLabel }}
      </UButton>
    </div>
  </UCard>
  <slot v-else />
</template>
