<script setup lang="ts">
defineProps<{
  modelValue: string;
  disabled?: boolean;
  id?: string;
  name?: string;
  ariaLabel?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
</script>
<template>
  <input
    class="form-control"
    type="month"
    :id="id"
    :name="name"
    :value="modelValue"
    :disabled="disabled"
    :aria-label="ariaLabel || '対象月'"
    required
    @change="
      ($event.target as HTMLInputElement).value &&
      emit('update:modelValue', ($event.target as HTMLInputElement).value)
    "
  />
</template>
