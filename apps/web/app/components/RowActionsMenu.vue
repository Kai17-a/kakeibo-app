<script setup lang="ts">
defineProps<{ label: string }>();
const emit = defineEmits<{ edit: []; delete: [] }>();
const id = useId();
const menu = ref<HTMLElement>();
const position = ref({ top: "0px", left: "0px" });
function locate(event: MouseEvent) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  position.value = {
    top: `${Math.max(8, Math.min(rect.bottom + 4, window.innerHeight - 120))}px`,
    left: `${Math.max(8, Math.min(rect.right - 160, window.innerWidth - 176))}px`,
  };
}
function select(action: "edit" | "delete") {
  menu.value?.hidePopover();
  if (action === "edit") emit("edit");
  else emit("delete");
}
</script>
<template>
  <span
    ><button
      type="button"
      class="btn btn-ghost-secondary btn-sm"
      :popovertarget="id"
      @click="locate"
      :aria-label="`${label}の操作`"
    >
      •••
    </button>
    <div :id="id" ref="menu" :style="position" popover class="app-row-menu ui-card">
      <button type="button" class="btn btn-ghost-secondary" @click="select('edit')">編集</button
      ><button type="button" class="btn btn-ghost-danger" @click="select('delete')">削除</button>
    </div></span
  >
</template>
