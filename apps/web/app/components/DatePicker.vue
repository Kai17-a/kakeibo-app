<script setup lang="ts">
const props = defineProps<{
  modelValue: string;
  min?: string;
  placeholder?: string;
  clearable?: boolean;
  disabled?: boolean;
  required?: boolean;
  id?: string;
  name?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const field = inject("app-field", undefined) as import("~/types/field").FieldContext | undefined;
</script>
<template>
  <div class="flex w-full">
    <input
      :id="id || field?.id"
      class="form-control min-w-0 flex-1"
      type="date"
      :name="name || field?.name"
      :value="props.modelValue"
      :min="min"
      :required="required || field?.required"
      :disabled="disabled"
      :aria-invalid="field?.error.value ? true : undefined"
      :aria-describedby="field?.error.value ? `${field.id}-error` : undefined"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
    <AppButton
      v-if="clearable && modelValue"
      variant="outline"
      color="neutral"
      aria-label="日付をクリア"
      :disabled="disabled"
      @click="emit('update:modelValue', '')"
      >×</AppButton
    >
  </div>
</template>
