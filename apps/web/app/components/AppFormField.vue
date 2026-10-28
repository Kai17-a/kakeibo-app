<script setup lang="ts">
import type { FieldContext } from "~/types/field";
const props = defineProps<{
  name?: string;
  label?: string;
  required?: boolean;
  error?: string | boolean;
  description?: string;
  hint?: string;
}>();
const id = useId();
const errors = inject<Ref<Record<string, string>>>("app-form-errors", ref({}));
const message = computed(() =>
  typeof props.error === "string" ? props.error : errors.value[props.name || ""] || "",
);
provide<FieldContext>("app-field", {
  id,
  get name() {
    return props.name;
  },
  get required() {
    return props.required;
  },
  error: message,
});
</script>
<template>
  <div class="field">
    <label v-if="label" :for="id"
      >{{ label }}<span v-if="required" aria-hidden="true"> *</span></label
    ><slot /><small v-if="$slots.help"><slot name="help" /></small
    ><small v-if="description || hint">{{ description || hint }}</small>
    <p v-if="message" :id="`${id}-error`" class="text-error" role="alert">{{ message }}</p>
  </div>
</template>
