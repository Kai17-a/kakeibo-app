<script setup lang="ts">
import type { FieldContext } from "~/types/field";
defineOptions({ inheritAttrs: false });
defineProps<{
  modelValue?: string | number | null;
  type?: string;
  disabled?: boolean;
  icon?: string;
  size?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const field = inject<FieldContext | undefined>("app-field", undefined);
</script>
<template>
  <input
    v-bind="$attrs"
    :id="($attrs.id as string) || field?.id"
    :name="($attrs.name as string) || field?.name"
    class="form-control"
    :type="type"
    :value="modelValue ?? ''"
    :disabled="disabled"
    :required="($attrs.required as boolean) || field?.required"
    :aria-invalid="field?.error.value ? true : undefined"
    :aria-describedby="field?.error.value ? `${field.id}-error` : undefined"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>
