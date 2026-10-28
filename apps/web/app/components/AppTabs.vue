<script setup lang="ts" generic="T extends string | number">
const props = defineProps<{
  modelValue: T;
  items: { label: string; value: string | number }[];
  orientation?: string;
  variant?: string;
  content?: boolean;
  ui?: unknown;
}>();
const emit = defineEmits<{ "update:modelValue": [value: T] }>();
function keydown(event: KeyboardEvent, index: number) {
  let next = index;
  if (event.key === "ArrowRight" || event.key === "ArrowDown") next++;
  else if (event.key === "ArrowLeft" || event.key === "ArrowUp") next--;
  else if (event.key === "Home") next = 0;
  else if (event.key === "End") next = props.items.length - 1;
  else return;
  event.preventDefault();
  next = (next + props.items.length) % props.items.length;
  emit("update:modelValue", props.items[next]!.value as T);
  const buttons = (
    event.currentTarget as HTMLElement
  ).parentElement?.querySelectorAll<HTMLButtonElement>("button");
  buttons?.item(next)?.focus();
}
</script>
<template>
  <div class="tabs" role="tablist" aria-label="表示切り替え">
    <button
      v-for="(item, index) in items"
      :key="item.value"
      type="button"
      role="tab"
      :aria-selected="modelValue === item.value"
      :tabindex="modelValue === item.value ? 0 : -1"
      @keydown="keydown($event, index)"
      @click="emit('update:modelValue', item.value as T)"
    >
      {{ item.label }}
    </button>
  </div>
</template>
