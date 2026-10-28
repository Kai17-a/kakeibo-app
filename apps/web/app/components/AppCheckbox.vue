<script setup lang="ts">
import type { FieldContext } from "~/types/field";
defineProps<{ modelValue?: boolean; label?: string; disabled?: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
const field = inject<FieldContext | undefined>("app-field", undefined);
</script>
<template>
  <label class="form-check"
    ><input
      :id="field?.id"
      class="form-check-input"
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    /><span>{{ label }}</span></label
  >
</template>
