<script setup lang="ts" generic="S extends object">
const props = defineProps<{
  state: S;
  validate: (state: S) => { name: string; message: string }[];
  disabled?: boolean;
}>();
const emit = defineEmits<{ submit: [] }>();
const errors = ref<Record<string, string>>({});
provide("app-form-errors", errors);
async function submit(event: Event) {
  if (props.disabled) return;
  errors.value = Object.fromEntries(
    props.validate(props.state).map((error) => [error.name, error.message]),
  );
  if (Object.keys(errors.value).length) {
    await nextTick();
    (event.target as HTMLFormElement).querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
    return;
  }
  emit("submit");
}
</script>
<template>
  <form novalidate @submit.prevent="submit" @input="errors = {}">
    <fieldset class="app-form-fields" :disabled="disabled"><slot /></fieldset>
  </form>
</template>
