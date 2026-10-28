<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    open?: boolean;
    title?: string;
    description?: string;
    dismissible?: boolean;
    close?: boolean;
    ui?: { content?: string; body?: string; footer?: string };
  }>(),
  { dismissible: true, close: true },
);
const emit = defineEmits<{ "update:open": [value: boolean]; "after:leave": [] }>();
const dialog = ref<HTMLDialogElement>();
const id = useId();
const modalStack = useState<string[]>("app-modal-stack", () => []);
let previous: HTMLElement | null = null;
watch(
  () => props.open,
  async (open) => {
    await nextTick();
    if (open && !dialog.value?.open) {
      previous = document.activeElement as HTMLElement;
      dialog.value?.showModal();
      modalStack.value = [...modalStack.value.filter((value) => value !== id), id];
    } else if (!open && dialog.value?.open) dialog.value.close();
  },
  { immediate: true },
);
function dismiss() {
  if (props.dismissible) emit("update:open", false);
}
function backdrop(event: MouseEvent) {
  const element = dialog.value;
  if (!element || event.target !== element) return;
  const box = element.getBoundingClientRect();
  if (
    event.clientX < box.left ||
    event.clientX > box.right ||
    event.clientY < box.top ||
    event.clientY > box.bottom
  )
    dismiss();
}
function closed() {
  modalStack.value = modalStack.value.filter((value) => value !== id);
  emit("update:open", false);
  previous?.focus();
  emit("after:leave");
}
onBeforeUnmount(() => {
  modalStack.value = modalStack.value.filter((value) => value !== id);
  dialog.value?.close();
  previous?.focus();
});
</script>
<template>
  <Teleport to="body"
    ><dialog
      ref="dialog"
      :id="id"
      class="ui-dialog app-dialog"
      :class="ui?.content"
      :aria-labelledby="`${id}-title`"
      :aria-describedby="description ? `${id}-description` : undefined"
      @cancel.prevent="dismiss"
      @click="backdrop"
      @close="closed"
    >
      <header class="flex items-start justify-between gap-4">
        <div>
          <h2 :id="`${id}-title`">{{ title }}</h2>
          <p v-if="description" :id="`${id}-description`" class="text-muted">{{ description }}</p>
        </div>
        <button
          v-if="close"
          type="button"
          class="btn btn-ghost-secondary"
          aria-label="閉じる"
          @click="emit('update:open', false)"
        >
          ×
        </button>
      </header>
      <div v-if="open" class="app-dialog-body" :class="ui?.body"><slot name="body" /><slot /></div>
      <footer v-if="open && $slots.footer" class="ui-card-footer justify-end" :class="ui?.footer">
        <slot name="footer" />
      </footer></dialog
  ></Teleport>
</template>
