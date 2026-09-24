<script setup lang="ts">
defineProps<{
  /** Names the row for assistive technology: rendered as 「{label}の操作」. */
  label: string;
}>();

const emit = defineEmits<{ edit: []; delete: [] }>();

const items = [
  [
    { label: "編集", icon: "i-lucide-pencil", onSelect: () => emit("edit") },
    {
      label: "削除",
      icon: "i-lucide-trash-2",
      color: "error" as const,
      onSelect: () => emit("delete"),
    },
  ],
];
</script>

<template>
  <UDropdownMenu :items="items">
    <UButton
      icon="i-lucide-ellipsis"
      color="neutral"
      variant="ghost"
      size="sm"
      :aria-label="`${label}の操作`"
    />
  </UDropdownMenu>
</template>
