<script setup lang="ts" generic="T extends string | number">
import type { FieldContext } from "~/types/field";
defineOptions({ inheritAttrs: false });
defineProps<{
  modelValue?: T | null;
  items?: (T | { label: string; value: T; disabled?: boolean })[];
  disabled?: boolean;
  placeholder?: string;
  size?: string;
  valueKey?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [value: T] }>();
const field = inject<FieldContext | undefined>("app-field", undefined);
</script>
<template>
  <select
    v-bind="$attrs"
    :id="($attrs.id as string) || field?.id"
    :name="($attrs.name as string) || field?.name"
    class="form-select"
    :disabled="disabled"
    :required="($attrs.required as boolean) || field?.required"
    :value="modelValue ?? ''"
    :aria-invalid="field?.error.value ? true : undefined"
    :aria-describedby="field?.error.value ? `${field.id}-error` : undefined"
    @change="
      emit(
        'update:modelValue',
        (items
          ?.map((item) => (typeof item === 'object' ? item.value : item))
          .find((value) => String(value) === ($event.target as HTMLSelectElement).value) ??
          '') as T,
      )
    "
  >
    <option v-if="placeholder || modelValue == null" value="" disabled>
      {{ placeholder || "選択してください" }}
    </option>
    <option
      v-for="(item, index) in items"
      :key="index"
      :value="typeof item === 'object' ? item.value : item"
      :disabled="typeof item === 'object' && item.disabled"
    >
      {{ typeof item === "object" ? item.label : item }}
    </option>
  </select>
</template>
