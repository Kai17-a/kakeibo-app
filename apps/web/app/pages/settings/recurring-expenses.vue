<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { Category, PaymentMethod, RecurringExpense } from "~/types/settings";
import { formatYen } from "~/utils/settings";
import {
  monthLabel,
  recurringExpenseForm,
  recurringExpenseInput,
  validateRecurringExpense,
} from "~/utils/recurring-expenses";

useSeoMeta({ title: "定期支出設定" });
const label = "定期支出";
const settingsApi = useSettingsApi();
const api = settingsApi.recurringExpenses;
const categories = ref<Category[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const pendingMonths = ref<Record<string, string[]>>({});
const backfilling = ref<Record<string, boolean>>({});
const state = reactive(recurringExpenseForm());
const categoryOptions = computed(() =>
  categories.value.map((item) => ({ label: item.name, value: item.id })),
);
const paymentMethodOptions = computed(() =>
  paymentMethods.value.map((item) => ({ label: item.name, value: item.id })),
);
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
    const [expenses, expenseCategories, methods] = await Promise.all([
      api.list(),
      settingsApi.categories("expense").list(),
      settingsApi.paymentMethods.list(),
    ]);
    categories.value = expenseCategories;
    paymentMethods.value = methods;
    await loadPendingMonths(expenses);
    return expenses;
  },
  create: api.create,
  update: api.update,
  remove: api.remove,
  afterSave: loadPendingMonths,
});
const sortedItems = computed(() =>
  items.value.toSorted((a, b) => a.name.localeCompare(b.name, "ja")),
);
const columns: TableColumn<RecurringExpense>[] = [
  {
    accessorKey: "name",
    header: "名称",
    meta: { class: { th: "w-full min-w-0", td: "w-full min-w-0" } },
  },
  {
    id: "category",
    header: "カテゴリ / 支払方法",
    meta: { class: { th: "hidden sm:table-cell", td: "hidden sm:table-cell" } },
  },
  {
    accessorKey: "payment_day",
    header: "支払日",
    cell: ({ row }) => `毎月${row.original.payment_day}日`,
    meta: {
      class: {
        th: "hidden whitespace-nowrap md:table-cell",
        td: "hidden whitespace-nowrap md:table-cell",
      },
    },
  },
  {
    accessorKey: "amount",
    header: "金額",
    cell: ({ row }) => formatYen(row.original.amount),
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
        th: "hidden whitespace-nowrap lg:table-cell",
        td: "hidden whitespace-nowrap lg:table-cell",
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
function paymentMethodName(id: string) {
  return paymentMethods.value.find((item) => item.id === id)?.name ?? "不明な支払方法";
}
function validate() {
  return validateRecurringExpense(state, categories.value, paymentMethods.value);
}
function openForm(item?: RecurringExpense) {
  Object.assign(state, recurringExpenseForm(item));
  openCrudForm(item);
}
const toast = useToast();
function save() {
  return saveItem(recurringExpenseInput(state));
}
async function loadPendingMonths(list: RecurringExpense[]) {
  const entries = await Promise.all(
    list.map(async (item) => [item.id, (await api.pendingMonths(item.id)).months] as const),
  );
  pendingMonths.value = Object.fromEntries(entries);
}
async function backfill(item: RecurringExpense) {
  if (backfilling.value[item.id]) return;
  backfilling.value = { ...backfilling.value, [item.id]: true };
  try {
    const result = await api.backfill(item.id);
    toast.add({
      title:
        result.created.length > 0
          ? `${result.created.length}か月分を計上しました`
          : "計上対象の月はありませんでした",
      color: "success",
    });
    await loadPendingMonths(items.value);
  } catch (error) {
    toast.add({
      title: "過去分を計上できませんでした",
      description: apiErrorMessage(error),
      color: "error",
    });
  } finally {
    backfilling.value = { ...backfilling.value, [item.id]: false };
  }
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="recurring-expense-heading">
    <SettingsListHeader
      heading-id="recurring-expense-heading"
      :title="label"
      description="毎月発生する固定費・準固定費（金額変動）の支出予定を管理します。"
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
      empty-description="毎月の固定費を登録すると、ここに一覧が表示されます。"
      add-label="定期支出を追加"
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
                  準固定費
                </UBadge>
              </div>
              <p
                v-if="row.original.description"
                class="mt-0.5 text-xs break-words whitespace-pre-wrap text-muted"
              >
                {{ row.original.description }}
              </p>
              <p class="mt-0.5 text-xs break-words text-muted sm:hidden">
                {{ categoryName(row.original.category_id) }} /
                {{ paymentMethodName(row.original.payment_method_id) }} · 毎月{{
                  row.original.payment_day
                }}日 · {{ row.original.is_active ? "有効" : "無効" }}
              </p>
            </div>
          </template>
          <template #category-cell="{ row }">
            <span class="text-sm break-words whitespace-normal text-muted">
              {{ categoryName(row.original.category_id) }} /
              {{ paymentMethodName(row.original.payment_method_id) }}
            </span>
          </template>
          <template #is_active-cell="{ row }">
            <UBadge :color="row.original.is_active ? 'success' : 'neutral'" variant="soft">
              {{ row.original.is_active ? "有効" : "無効" }}
            </UBadge>
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
      <div class="flex flex-col gap-3">
        <template v-for="item in sortedItems" :key="item.id">
          <div
            v-if="(pendingMonths[item.id]?.length ?? 0) > 0"
            class="flex flex-wrap items-center gap-3 rounded-md bg-elevated/50 px-4 py-3 text-sm"
          >
            <span class="font-medium">{{ item.name }}</span>
            <UBadge color="neutral" variant="soft">
              未計上 {{ pendingMonths[item.id]?.length }}件
            </UBadge>
            <UButton
              v-if="!item.is_variable"
              color="neutral"
              variant="outline"
              size="sm"
              :loading="backfilling[item.id]"
              :disabled="backfilling[item.id]"
              @click="backfill(item)"
            >
              まとめて計上する
            </UButton>
            <details v-else class="w-full">
              <summary class="cursor-pointer text-muted">過去月を登録</summary>
              <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1">
                <a
                  v-for="month in pendingMonths[item.id]"
                  :key="month"
                  class="text-primary underline underline-offset-4"
                  :href="`/monthly?month=${month}`"
                >
                  {{ monthLabel(month) }}
                </a>
              </div>
            </details>
          </div>
        </template>
      </div>
    </SettingsCollectionState>

    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="毎月の支払額、支払日、適用期間を設定します。"
      form-id="recurring-expense-form"
      :state="state"
      :validate="validate"
      :saving="saving"
      :error="formError"
      wide
      @submit="save"
    >
      <UAlert
        v-if="!categories.length || !paymentMethods.length"
        color="warning"
        title="カテゴリ・支払方法が不足しています"
        description="設定の「支出カテゴリ」「支払方法」で登録してから追加してください。"
      />
      <UFormField name="name" label="名称" required>
        <UInput v-model="state.name" class="w-full" />
      </UFormField>
      <UFormField name="usdBased">
        <UCheckbox v-model="state.usdBased" label="外貨建て（USD）にする" />
      </UFormField>
      <UFormField
        v-if="!state.usdBased"
        name="amount"
        :label="state.is_variable ? '金額（目安）' : '金額'"
        required
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
      <UFormField
        v-else
        name="foreignAmount"
        label="毎月のUSD金額"
        description="実際の引き落とし額は毎月の為替レートで自動計算されます。"
        required
      >
        <UInput
          :model-value="state.foreignAmount"
          type="number"
          :min="0.01"
          step="any"
          class="w-full"
          @update:model-value="state.foreignAmount = String($event ?? '')"
        />
      </UFormField>
      <UFormField name="payment_day" label="毎月の支払日" required>
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
      <UFormField name="payment_method_id" label="支払方法" required>
        <USelect
          v-model="state.payment_method_id"
          :items="paymentMethodOptions"
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
        <UCheckbox v-model="state.is_variable" label="金額が月ごとに変動する（準固定費）" />
      </UFormField>
      <UFormField v-if="editingId" name="sync_future_transactions">
        <UCheckbox
          v-model="state.sync_future_transactions"
          label="今月以降に生成済みの明細へ金額・カテゴリ・支払方法・備考を反映する"
        />
      </UFormField>
      <UFormField name="is_active">
        <UCheckbox
          v-model="state.is_active"
          :label="editingId ? '有効にする' : '登録後すぐに有効にする'"
        />
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
