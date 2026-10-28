<script setup lang="ts">
const { toasts, remove } = useAppToast();
const modalStack = useState<string[]>("app-modal-stack", () => []);
const target = computed(() => {
  const id = modalStack.value.at(-1);
  return (id && document.getElementById(id)) || "body";
});
</script>
<template>
  <Teleport :to="target"
    ><div class="ui-toasts" aria-label="通知">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="ui-toast alert"
        :class="`alert-${toast.color === 'error' ? 'danger' : toast.color || 'info'}`"
        :role="toast.color === 'error' ? 'alert' : 'status'"
      >
        <div class="ui-toast-content">
          <div class="ui-toast-title">{{ toast.title }}</div>
          <p v-if="toast.description" class="ui-toast-body">{{ toast.description }}</p>
        </div>
        <button
          class="btn btn-ghost-secondary"
          type="button"
          aria-label="通知を閉じる"
          @click="remove(toast.id)"
        >
          ×
        </button>
      </div>
    </div></Teleport
  >
</template>
