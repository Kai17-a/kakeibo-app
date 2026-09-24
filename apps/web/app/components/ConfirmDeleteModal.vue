<script setup lang="ts">
withDefaults(
  defineProps<{
    /** What will be deleted, e.g. 「この支出明細を削除しますか？」. */
    description: string;
    /** True while the deletion request is in flight; locks the dialog. */
    busy?: boolean;
    title?: string;
  }>(),
  { busy: false, title: "削除の確認" },
);

const open = defineModel<boolean>("open", { required: true });
const emit = defineEmits<{ confirm: [] }>();
</script>

<template>
  <UModal
    v-model:open="open"
    :title="title"
    :description="description"
    :dismissible="!busy"
    :close="!busy"
    :ui="{ footer: 'justify-end' }"
  >
    <template v-if="$slots.body" #body>
      <slot name="body" />
    </template>
    <template #footer>
      <UButton color="neutral" variant="outline" :disabled="busy" @click="open = false">
        キャンセル
      </UButton>
      <UButton color="error" :loading="busy" :disabled="busy" @click="emit('confirm')">
        削除
      </UButton>
    </template>
  </UModal>
</template>
