<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { Category, RecurringIncome } from "~/types/settings";
import { formatYen } from "~/utils/settings";
import {
  recurringIncomeForm,
  recurringIncomeInput,
  validateRecurringIncome,
} from "~/utils/recurring-incomes";

useSeoMeta({ title: "定期収入設定" });
const label = "定期収入";
const settingsApi = useSettingsApi();
const api = settingsApi.recurringIncomes;
const categories = ref<Category[]>([]);
const categoryOptions = computed(() =>
  categories.value.map((item) => ({ label: item.name, value: item.id })),
);
const state = reactive(recurringIncomeForm());
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
    const [incomes, incomeCategories] = await Promise.all([
      api.list(),
      settingsApi.categories("income").list(),
    ]);
    categories.value = incomeCategories;
    return incomes;
  },
  create: api.create,
  update: api.update,
  remove: api.remove,
});
const sortedItems = computed(() =>
  items.value.toSorted((a, b) => a.name.localeCompare(b.name, "ja")),
);
const columns: TableColumn<RecurringIncome>[] = [
  {
    accessorKey: "name",
    header: "名称",
    meta: { class: { th: "w-full min-w-0", td: "w-full min-w-0" } },
  },
  {
    accessorKey: "category_id",
    header: "カテゴリ名",
    cell: ({ row }) =>
      categories.value.find((item) => item.id === row.original.category_id)?.name ??
      "不明なカテゴリ",
    meta: { class: { th: "hidden w-40 sm:table-cell", td: "hidden w-40 sm:table-cell" } },
  },
  {
    accessorKey: "payment_day",
    header: "入金日",
    cell: ({ row }) => `毎月${row.original.payment_day}日`,
    meta: {
      class: {
        th: "hidden w-24 whitespace-nowrap md:table-cell",
        td: "hidden w-24 whitespace-nowrap md:table-cell",
      },
    },
  },
  {
    accessorKey: "amount",
    header: "金額",
    meta: {
      class: {
        th: "w-24 whitespace-nowrap text-right",
        td: "w-24 whitespace-nowrap text-right tabular-nums",
      },
    },
  },
  {
    accessorKey: "is_active",
    header: "状態",
    meta: {
      class: {
        th: "hidden w-20 whitespace-nowrap lg:table-cell",
        td: "hidden w-20 whitespace-nowrap lg:table-cell",
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
function validate() {
  return validateRecurringIncome(state, categories.value);
}

function openForm(item?: RecurringIncome) {
  Object.assign(state, recurringIncomeForm(item));
  openCrudForm(item);
}
function save() {
  return saveItem(recurringIncomeInput(state));
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="recurring-income-heading">
    <SettingsListHeader
      heading-id="recurring-income-heading"
      :title="label"
      description="毎月発生する固定収入・準固定収入（金額変動）の入金予定を管理します。"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError"
      @add="openForm()"
    />

    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-repeat"
      empty-description="毎月の定期収入を登録すると、ここに一覧が表示されます。"
      add-label="定期収入を追加"
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
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <span class="font-medium break-words whitespace-normal text-default">{{
                  row.original.name
                }}</span>
                <UBadge v-if="row.original.is_variable" color="neutral" variant="soft">
                  準固定収入
                </UBadge>
              </div>
              <p
                v-if="row.original.description"
                class="mt-0.5 text-xs break-words whitespace-pre-wrap text-muted"
              >
                {{ row.original.description }}
              </p>
              <p class="mt-0.5 text-xs break-words text-muted sm:hidden">
                {{
                  categories.find((item) => item.id === row.original.category_id)?.name ??
                  "不明なカテゴリ"
                }}
                · 毎月{{ row.original.payment_day }}日 ·
                {{ row.original.is_active ? "有効" : "無効" }}
              </p>
            </div>
          </template>
          <template #is_active-cell="{ row }">
            <UBadge :color="row.original.is_active ? 'success' : 'neutral'" variant="soft">
              {{ row.original.is_active ? "有効" : "無効" }}
            </UBadge>
          </template>
          <template #amount-cell="{ row }">
            <span :class="row.original.amount === null ? 'text-muted' : ''">
              {{ row.original.amount === null ? "毎月入力" : formatYen(row.original.amount) }}
            </span>
          </template>
          <template #actions-cell="{ row }">
            <RowActionsMenu
              :label="row.original.name"
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
      description="毎月の収入額、入金日、適用期間を設定します。"
      form-id="recurring-income-form"
      :state="state"
      :validate="validate"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UAlert
        v-if="!categories.length"
        color="warning"
        title="収入カテゴリがありません"
        description="設定の「収入カテゴリ」でカテゴリを登録してから追加してください。"
      />
      <UFormField name="name" label="名称" required>
        <UInput v-model="state.name" class="w-full" autofocus />
      </UFormField>
      <UFormField
        name="amount"
        :label="state.is_variable ? '金額（目安・任意）' : '金額'"
        :required="!state.is_variable"
        :description="
          state.is_variable ? '空欄の場合は「今月分を登録」で毎月入力します。' : undefined
        "
      >
        <UInput
          :model-value="state.amount"
          type="number"
          :min="1"
          :step="1"
          class="w-full"
          @update:model-value="state.amount = String($event ?? '')"
        />
      </UFormField>
      <UFormField name="payment_day" label="毎月の入金日" required>
        <UInput
          :model-value="state.payment_day"
          type="number"
          :min="1"
          :max="31"
          :step="1"
          class="w-full"
          @update:model-value="state.payment_day = String($event ?? '')"
        />
      </UFormField>
      <UFormField name="category_id" label="カテゴリ" required>
        <USelect
          v-model="state.category_id"
          :items="categoryOptions"
          placeholder="選択してください"
          class="w-full"
        />
      </UFormField>
      <UFormField name="start_date" label="開始日" required>
        <DatePicker v-model="state.start_date" required class="w-full" />
      </UFormField>
      <UFormField name="end_date" label="終了日（任意）">
        <DatePicker v-model="state.end_date" :min="state.start_date" clearable class="w-full" />
      </UFormField>
      <UFormField name="description" label="備考（任意）">
        <UTextarea v-model="state.description" class="w-full" />
      </UFormField>
      <UFormField name="is_variable">
        <UCheckbox v-model="state.is_variable" label="金額が月ごとに変動する（準固定収入）" />
      </UFormField>
      <UFormField name="is_active">
        <UCheckbox v-model="state.is_active" label="有効にする" />
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
