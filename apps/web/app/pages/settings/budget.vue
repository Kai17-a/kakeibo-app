<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { Budget, Category } from "~/types/settings";
import {
  availableBudgetCategories,
  formatYen,
  groupCategories,
  validAmount,
} from "~/utils/settings";

useSeoMeta({ title: "予算設定" });
const label = "予算";
const settingsApi = useSettingsApi();
const api = settingsApi.budgets;
const categories = ref<Category[]>([]);
const state = reactive<{ category_id: string; amount: string }>({ category_id: "", amount: "" });

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
  fetch: async () => {
    const [budgets, expenseCategories] = await Promise.all([
      api.list(),
      settingsApi.categories("expense").list(),
    ]);
    categories.value = groupCategories(expenseCategories);
    return budgets;
  },
  create: api.create,
  update: api.update,
  remove: api.remove,
  removeErrorMessage: () => "予算を削除できませんでした。",
});

/** Expense categories without a budget yet; a category has at most one budget. */
const unassignedCategories = computed(() =>
  availableBudgetCategories(categories.value, items.value),
);
const budgetCategories = computed(() =>
  availableBudgetCategories(
    categories.value,
    items.value,
    items.value.find((item) => item.id === editingId.value)?.category_id,
  ),
);
const categoryOptions = computed(() =>
  budgetCategories.value.map((item) => ({ label: item.name, value: item.id })),
);
const columns: TableColumn<Budget>[] = [
  {
    accessorKey: "category_id",
    header: "カテゴリ名",
    cell: ({ row }) => categoryName(row.original.category_id),
    meta: { class: { th: "w-full min-w-0", td: "w-full min-w-0" } },
  },
  {
    accessorKey: "amount",
    header: "月額予算",
    cell: ({ row }) => formatYen(row.original.amount),
    meta: {
      class: {
        th: "w-28 whitespace-nowrap text-right",
        td: "w-28 whitespace-nowrap text-right tabular-nums",
      },
    },
  },
  {
    id: "actions",
    header: "操作",
    meta: {
      class: { th: "w-14 whitespace-nowrap text-right", td: "w-14 whitespace-nowrap text-right" },
    },
  },
];
function categoryName(id: string) {
  return categories.value.find((item) => item.id === id)?.name ?? "不明なカテゴリ";
}
function validate() {
  const errors: { name: string; message: string }[] = [];
  if (!budgetCategories.value.some((item) => item.id === state.category_id))
    errors.push({ name: "category_id", message: "支出カテゴリを選択してください。" });
  if (!validAmount(state.amount))
    errors.push({ name: "amount", message: "月額予算を0以上の整数で入力してください。" });
  return errors;
}
function openForm(item?: Budget) {
  if (!item && !unassignedCategories.value.length) return;
  Object.assign(state, { category_id: item?.category_id ?? "", amount: item?.amount ?? "" });
  openCrudForm(item);
}
function save() {
  return saveItem({ category_id: state.category_id, amount: String(state.amount).trim() });
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="budget-heading">
    <SettingsListHeader
      heading-id="budget-heading"
      :title="label"
      description="支出カテゴリごとに毎月適用する予算を管理します。"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError || !unassignedCategories.length"
      @add="openForm()"
    />
    <p v-if="!loading && !loadError && !unassignedCategories.length" class="text-sm text-muted">
      予算未設定の支出カテゴリがありません。カテゴリを追加するか、登録済みの予算を編集してください。
    </p>
    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-chart-pie"
      empty-description="カテゴリごとの月額予算を設定すると、ここに一覧が表示されます。"
      add-label="予算を追加"
      :add-disabled="!unassignedCategories.length"
      @retry="load"
      @add="openForm()"
    >
      <div class="min-w-0 overflow-hidden rounded-lg border border-default">
        <UTable
          :data="items"
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
          <template #category_id-cell="{ row }">
            <span class="font-medium break-words text-default">{{
              categoryName(row.original.category_id)
            }}</span>
          </template>
          <template #actions-cell="{ row }">
            <RowActionsMenu
              :label="categoryName(row.original.category_id)"
              @edit="openForm(row.original)"
              @delete="askDelete(row.original)"
            />
          </template>
        </UTable>
      </div>
    </SettingsCollectionState>
    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="支出カテゴリと毎月の予算額を設定します。"
      form-id="budget-form"
      :state="state"
      :validate="validate"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UFormField name="category_id" label="支出カテゴリ" required>
        <USelect
          v-model="state.category_id"
          :items="categoryOptions"
          placeholder="選択してください"
          class="w-full"
        />
      </UFormField>
      <UFormField name="amount" label="月額予算" required>
        <UInput
          :model-value="state.amount"
          type="number"
          :min="0"
          :step="1"
          class="w-full"
          @update:model-value="state.amount = String($event ?? '')"
        />
      </UFormField>
    </SettingsFormModal>
    <SettingsDeleteDialog
      v-model:open="deleteOpen"
      :label="label"
      :target="deleting ? categoryName(deleting.category_id) : ''"
      :busy="removing"
      :error="deleteError"
      @confirm="remove"
    />
  </section>
</template>
