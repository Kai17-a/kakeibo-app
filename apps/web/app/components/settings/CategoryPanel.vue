<script setup lang="ts">
import SortableJs from "sortablejs";
import type { Category, CategoryKind } from "~/types/settings";
import { categoryMove, groupCategories, validateNamed } from "~/utils/settings";

const props = defineProps<{ kind: CategoryKind }>();
const label = computed(() => (props.kind === "expense" ? "支出カテゴリ" : "収入カテゴリ"));
const api = useSettingsApi().categories(props.kind);
const toast = useToast();
const reordering = ref(false);
const orderError = ref("");
const state = reactive({ name: "", description: "", parent_category_id: "none" });
const {
  items,
  loading,
  loadError,
  load,
  formOpen,
  editingId,
  saving,
  formError,
  openForm: openCrudForm,
  save: saveItem,
  deleteOpen,
  deleting,
  removing,
  deleteError,
  askDelete,
  remove,
} = useCrudCollection({
  label,
  fetch: api.list,
  create: api.create,
  update: api.update,
  remove: api.remove,
  removeErrorMessage: () =>
    "カテゴリを削除できませんでした。登録済みの明細で使用されているか、子カテゴリが存在するため削除できません。",
});
const sortedItems = computed(() => groupCategories(items.value));
const editingHasChildren = computed(
  () =>
    !!editingId.value && items.value.some((item) => item.parent_category_id === editingId.value),
);
const parentOptions = computed(() => [
  { label: "親カテゴリなし", value: "none" },
  ...sortedItems.value
    .filter((item) => item.parent_category_id === null && item.id !== editingId.value)
    .map((item) => ({ label: item.name, value: item.id })),
]);
const validate = validateNamed;
function openForm(item?: Category) {
  Object.assign(state, {
    name: item?.name ?? "",
    description: item?.description ?? "",
    parent_category_id: item?.parent_category_id ?? "none",
  });
  openCrudForm(item);
}
function parentName(item: Category) {
  return items.value.find((candidate) => candidate.id === item.parent_category_id)?.name;
}
async function move(item: Category, direction: -1 | 1) {
  if (reordering.value) return;
  const result = categoryMove(items.value, item.id, direction);
  if (!result) return;
  await reorder(result.input.parent_category_id, result.input.category_ids);
}
async function reorder(parent: string | null, ids: string[]) {
  if (reordering.value) return;
  const previous = items.value;
  items.value = items.value.map((item) => ({
    ...item,
    display_order: ids.includes(item.id) ? ids.indexOf(item.id) : item.display_order,
  }));
  reordering.value = true;
  orderError.value = "";
  try {
    await api.reorder({ parent_category_id: parent, category_ids: ids });
    toast.add({ title: "カテゴリの表示順を更新しました", color: "success" });
  } catch (error) {
    items.value = previous;
    orderError.value = apiErrorMessage(error);
  } finally {
    reordering.value = false;
  }
}
const list = ref<HTMLElement | null>(null);
let sortable: SortableJs | undefined;
watch(
  [list, reordering],
  () => {
    sortable?.destroy();
    if (!list.value) return;
    sortable = SortableJs.create(list.value, {
      handle: ".category-handle",
      draggable: ".category-row",
      disabled: reordering.value,
      animation: 150,
      delay: 150,
      delayOnTouchOnly: true,
      touchStartThreshold: 5,
      onMove: (event) => event.dragged.dataset.parent === event.related.dataset.parent,
      onEnd: (event) => {
        const id = event.item.dataset.id;
        const item = items.value.find((entry) => entry.id === id);
        const container = event.from;
        const ids = Array.from(container.children)
          .filter((el) => (el as HTMLElement).dataset.parent === event.item.dataset.parent)
          .map((el) => (el as HTMLElement).dataset.id!);
        // Restore Vue's DOM order before applying the reactive update.
        const children = Array.from(container.children).filter((el) => el !== event.item);
        container.insertBefore(event.item, children[event.oldIndex ?? 0] ?? null);
        if (item && event.oldIndex !== event.newIndex) void reorder(item.parent_category_id, ids);
      },
    });
  },
  { flush: "post" },
);
onBeforeUnmount(() => sortable?.destroy());
function save() {
  return saveItem({
    name: state.name.trim(),
    description: state.description.trim() || null,
    parent_category_id:
      editingHasChildren.value || state.parent_category_id === "none"
        ? null
        : state.parent_category_id,
  });
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="category-heading">
    <SettingsListHeader
      heading-id="category-heading"
      :title="label"
      description="収支の登録時に選択するカテゴリと表示順を管理します"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError || reordering"
      @add="openForm()"
    />

    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-tags"
      empty-description="収支を整理するカテゴリを登録すると、ここに一覧が表示されます"
      add-label="カテゴリを追加"
      @retry="load"
      @add="openForm()"
    >
      <div
        ref="list"
        class="divide-y divide-default overflow-hidden rounded-lg border border-default"
        :aria-label="`${label}一覧`"
      >
        <div
          v-for="item in sortedItems"
          :key="item.id"
          class="category-row flex cursor-pointer items-center gap-2 px-3 py-2 text-sm hover:bg-elevated/50"
          :data-id="item.id"
          :data-parent="item.parent_category_id ?? ''"
          @click="openForm(item)"
        >
          <button
            type="button"
            class="category-handle flex size-7 shrink-0 touch-none items-center justify-center rounded focus-visible:ring-2 focus-visible:ring-primary max-sm:size-[44px]"
            :aria-label="`${item.name}の並べ替え`"
            :disabled="reordering"
            @click.stop
            @keydown.up.prevent.stop="move(item, -1)"
            @keydown.down.prevent.stop="move(item, 1)"
          >
            <UIcon name="i-lucide-grip-vertical" class="size-5 text-muted" />
          </button>
          <div :class="['min-w-0 flex-1', item.parent_category_id ? 'pl-6' : '']">
            <button
              type="button"
              class="text-left font-medium break-words text-default focus-visible:ring-2 focus-visible:ring-primary max-sm:min-h-[44px]"
              :aria-label="`${item.name}を編集`"
              @click.stop="openForm(item)"
            >
              {{ item.name }}
            </button>
            <p
              v-if="item.parent_category_id || item.description"
              class="text-xs break-words whitespace-pre-wrap text-muted"
            >
              <span v-if="item.parent_category_id">親: {{ parentName(item) }} </span
              >{{ item.description }}
            </p>
          </div>
          <div @click.stop>
            <RowActionsMenu :label="item.name" @edit="openForm(item)" @delete="askDelete(item)" />
          </div>
        </div>
      </div>
      <UAlert
        v-if="orderError"
        color="error"
        title="表示順を更新できませんでした"
        :description="orderError"
      />
    </SettingsCollectionState>
    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="カテゴリ名、親カテゴリ、説明を設定します"
      form-id="category-form"
      :state="state"
      :validate="validate"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UFormField name="name" label="カテゴリ名" required>
        <UInput v-model="state.name" :maxlength="100" class="w-full" autofocus />
      </UFormField>
      <UFormField
        name="parent_category_id"
        label="親カテゴリ（任意）"
        :description="
          editingHasChildren
            ? '子カテゴリが存在するため、このカテゴリには親を設定できません'
            : undefined
        "
      >
        <USelect
          v-model="state.parent_category_id"
          :items="parentOptions"
          :disabled="editingHasChildren"
          class="w-full"
        />
      </UFormField>
      <UFormField name="description" label="説明（任意）">
        <UTextarea v-model="state.description" :maxlength="500" class="w-full" />
      </UFormField>
    </SettingsFormModal>
    <SettingsDeleteDialog
      v-model:open="deleteOpen"
      :label="label"
      :target="deleting?.name ?? ''"
      :busy="removing"
      :error="deleteError"
      @confirm="remove"
    />
  </section>
</template>
