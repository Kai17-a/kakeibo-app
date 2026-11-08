<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
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
const columns: TableColumn<Category>[] = [
  {
    accessorKey: "name",
    header: "カテゴリ名",
    meta: { class: { th: "w-full min-w-0", td: "w-full min-w-0" } },
  },
  {
    id: "actions",
    header: "操作",
    meta: {
      class: { th: "w-32 whitespace-nowrap text-right", td: "w-32 whitespace-nowrap text-right" },
    },
  },
];
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
function canMove(item: Category, direction: -1 | 1) {
  return categoryMove(items.value, item.id, direction) !== null;
}
async function move(item: Category, direction: -1 | 1) {
  if (reordering.value) return;
  const result = categoryMove(items.value, item.id, direction);
  if (!result) return;
  const previous = items.value;
  items.value = result.items;
  reordering.value = true;
  orderError.value = "";
  try {
    await api.reorder(result.input);
    toast.add({ title: "カテゴリの表示順を更新しました", color: "success" });
  } catch (error) {
    items.value = previous;
    orderError.value = apiErrorMessage(error);
  } finally {
    reordering.value = false;
  }
}
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
      description="収支の登録時に選択するカテゴリと表示順を管理します。"
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
      empty-description="収支を整理するカテゴリを登録すると、ここに一覧が表示されます。"
      add-label="カテゴリを追加"
      @retry="load"
      @add="openForm()"
    >
      <div class="min-w-0 overflow-hidden rounded-lg border border-default">
        <UTable
          :data="sortedItems"
          :columns="columns"
          :aria-label="`${label}一覧`"
          class="w-full"
          :ui="{
            root: 'overflow-hidden',
            base: 'w-full table-fixed',
            th: 'px-3 py-2 sm:px-4',
            td: 'px-3 py-2 whitespace-normal sm:px-4',
          }"
        >
          <template #name-cell="{ row }">
            <div :class="['min-w-0', row.original.parent_category_id !== null ? 'pl-6' : '']">
              <p class="font-medium break-words whitespace-normal text-default">
                {{ row.original.name }}
              </p>
              <p
                v-if="row.original.parent_category_id !== null || row.original.description"
                class="mt-0.5 text-xs whitespace-normal text-muted"
              >
                <span v-if="row.original.parent_category_id !== null"
                  >親: {{ parentName(row.original) }}</span
                >
                <span v-if="row.original.parent_category_id !== null && row.original.description">
                  ·
                </span>
                <span class="break-words whitespace-pre-wrap">{{ row.original.description }}</span>
              </p>
            </div>
          </template>
          <template #actions-cell="{ row }">
            <div class="flex items-center justify-end gap-1">
              <UButton
                icon="i-lucide-arrow-up"
                color="neutral"
                variant="ghost"
                size="sm"
                :aria-label="`${row.original.name}を上へ移動`"
                :disabled="reordering || !canMove(row.original, -1)"
                @click="move(row.original, -1)"
              />
              <UButton
                icon="i-lucide-arrow-down"
                color="neutral"
                variant="ghost"
                size="sm"
                :aria-label="`${row.original.name}を下へ移動`"
                :disabled="reordering || !canMove(row.original, 1)"
                @click="move(row.original, 1)"
              />
              <RowActionsMenu
                :label="row.original.name"
                @edit="openForm(row.original)"
                @delete="askDelete(row.original)"
              />
            </div>
          </template>
        </UTable>
      </div>
    </SettingsCollectionState>
    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="カテゴリ名、親カテゴリ、説明を設定します。"
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
            ? '子カテゴリが存在するため、このカテゴリには親を設定できません。'
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
