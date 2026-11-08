<script setup lang="ts" generic="S extends object">
withDefaults(
  defineProps<{
    /** The title: 「{label}を追加」 or 「{label}を編集」. */
    label: string;
    editing: boolean;
    description: string;
    /** The form element id, linking the footer submit button to the form. */
    formId: string;
    state: S;
    validate: (state: S) => { name: string; message: string }[];
    saving: boolean;
    /** Why the last save failed; empty when it hasn't failed. */
    error: string;
    /** Widens the dialog for forms with many fields. */
    wide?: boolean;
  }>(),
  { wide: false },
);

const open = defineModel<boolean>("open", { required: true });
const emit = defineEmits<{ submit: [] }>();
</script>

<template>
  <UModal
    v-model:open="open"
    :title="`${label}を${editing ? '編集' : '追加'}`"
    :description="description"
    :dismissible="!saving"
    :close="!saving"
    :ui="wide ? { footer: 'justify-end', content: 'max-w-2xl' } : { footer: 'justify-end' }"
  >
    <template #body>
      <UForm
        :id="formId"
        :state="state"
        :validate="validate"
        :disabled="saving"
        class="space-y-5"
        @submit="emit('submit')"
      >
        <UAlert v-if="error" color="error" title="保存できませんでした" :description="error" />
        <slot />
      </UForm>
    </template>
    <template #footer>
      <UButton color="neutral" variant="outline" :disabled="saving" @click="open = false">
        キャンセル
      </UButton>
      <UButton type="submit" :form="formId" :loading="saving" :disabled="saving">
        {{ editing ? "更新" : "追加" }}
      </UButton>
    </template>
  </UModal>
</template>
