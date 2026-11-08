<script setup lang="ts">
withDefaults(
  defineProps<{
    /** The collection name, used in the title 「{label}を削除」. */
    label: string;
    /** The name of the item being deleted. */
    target: string;
    busy: boolean;
    /** Why the last delete attempt failed; empty when it hasn't failed. */
    error: string;
    description?: string;
  }>(),
  { description: "この設定を削除します。この操作は取り消せません。" },
);

const open = defineModel<boolean>("open", { required: true });
const emit = defineEmits<{ confirm: [] }>();
</script>

<template>
  <ConfirmDeleteModal
    v-model:open="open"
    :title="`${label}を削除`"
    :description="description"
    :busy="busy"
    @confirm="emit('confirm')"
  >
    <template #body>
      <p class="break-all">
        {{ target }}
      </p>
      <UAlert
        v-if="error"
        color="error"
        title="削除できませんでした"
        :description="error"
        class="mt-4"
      />
    </template>
  </ConfirmDeleteModal>
</template>
