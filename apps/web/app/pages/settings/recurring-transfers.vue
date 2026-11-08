<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { PaymentMethod, RecurringTransfer } from "~/types/settings";
import { formatYen } from "~/utils/settings";
import {
  recurringTransferForm,
  recurringTransferInput,
  validateRecurringTransfer,
} from "~/utils/recurring-transfers";
import {
  formatRecurringSchedule,
  setRecurringMonthEnd,
  updateRecurringPaymentDay,
} from "~/utils/recurring-payment-day";

useSeoMeta({ title: "定期振替設定" });
const label = "定期振替";
const settingsApi = useSettingsApi();
const api = settingsApi.recurringTransfers;
const paymentMethods = ref<PaymentMethod[]>([]);
const paymentOptions = computed(() =>
  paymentMethods.value.map((item) => ({ label: item.name, value: item.id })),
);
const paymentNames = computed(
  () => new Map(paymentMethods.value.map((item) => [item.id, item.name])),
);
const state = reactive(recurringTransferForm());
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
    const [transfers, methods] = await Promise.all([api.list(), settingsApi.paymentMethods.list()]);
    paymentMethods.value = methods;
    return transfers;
  },
  create: api.create,
  update: api.update,
  remove: api.remove,
});
const sortedItems = computed(() =>
  items.value.toSorted((a, b) => a.name.localeCompare(b.name, "ja")),
);
const columns: TableColumn<RecurringTransfer>[] = [
  { accessorKey: "name", header: "名称", meta: { class: { th: "w-full", td: "w-full" } } },
  {
    accessorKey: "payment_day",
    header: "振替日",
    cell: ({ row }) => formatRecurringSchedule(row.original.payment_day),
    meta: { class: { th: "hidden w-24 md:table-cell", td: "hidden w-24 md:table-cell" } },
  },
  {
    accessorKey: "amount",
    header: "金額",
    meta: { class: { th: "w-28 text-right", td: "w-28 text-right tabular-nums" } },
  },
  {
    accessorKey: "is_active",
    header: "状態",
    meta: { class: { th: "hidden w-20 lg:table-cell", td: "hidden w-20 lg:table-cell" } },
  },
  {
    id: "actions",
    header: "操作",
    meta: { class: { th: "w-14 text-right", td: "w-14 text-right" } },
  },
];
function openForm(item?: RecurringTransfer) {
  Object.assign(state, recurringTransferForm(item));
  openCrudForm(item);
}
function save() {
  return saveItem(recurringTransferInput(state));
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="recurring-transfer-heading">
    <SettingsListHeader
      heading-id="recurring-transfer-heading"
      :title="label"
      description="毎月決まった金額を支払方法間で移動する予定を管理します。"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError || paymentMethods.length < 2"
      @add="openForm()"
    />
    <UAlert
      v-if="!loading && !loadError && paymentMethods.length < 2"
      color="warning"
      title="支払方法が2件以上必要です"
      description="設定の「支払方法」で移動元と移動先を登録してから定期振替を追加してください。"
    />
    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-repeat-2"
      empty-description="毎月の定期振替を登録すると、ここに一覧が表示されます。"
      add-label="定期振替を追加"
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
              <p class="font-medium break-words text-default">{{ row.original.name }}</p>
              <p class="mt-0.5 text-xs break-words text-muted">
                {{ paymentNames.get(row.original.from_payment_method_id) ?? "不明" }} →
                {{ paymentNames.get(row.original.to_payment_method_id) ?? "不明" }}
                <span class="md:hidden">
                  · {{ formatRecurringSchedule(row.original.payment_day) }}</span
                >
              </p>
              <p
                v-if="row.original.description"
                class="mt-0.5 text-xs break-words whitespace-pre-wrap text-muted"
              >
                {{ row.original.description }}
              </p>
            </div>
          </template>
          <template #amount-cell="{ row }">{{ formatYen(row.original.amount) }}</template>
          <template #is_active-cell="{ row }"
            ><UBadge :color="row.original.is_active ? 'success' : 'neutral'" variant="soft">{{
              row.original.is_active ? "有効" : "無効"
            }}</UBadge></template
          >
          <template #actions-cell="{ row }"
            ><RowActionsMenu
              :label="row.original.name"
              @edit="openForm(row.original)"
              @delete="askDelete(row.original)"
          /></template>
        </UTable>
      </div>
    </SettingsCollectionState>
    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="毎月の振替額、振替日、適用期間を設定します。"
      form-id="recurring-transfer-form"
      :state="state"
      :validate="() => validateRecurringTransfer(state, paymentMethods)"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UFormField name="name" label="名称" required
        ><UInput v-model="state.name" class="w-full" autofocus
      /></UFormField>
      <UFormField name="amount" label="金額" required
        ><UInput
          :model-value="state.amount"
          type="number"
          :min="1"
          :step="1"
          class="w-full"
          @update:model-value="state.amount = String($event ?? '')"
      /></UFormField>
      <UFormField name="payment_day" label="毎月の振替日" required
        ><UInput
          :model-value="state.payment_day"
          type="number"
          :min="1"
          :max="31"
          :step="1"
          :disabled="state.is_month_end"
          class="w-full"
          @update:model-value="updateRecurringPaymentDay(state, $event)"
      /></UFormField>
      <UFormField name="is_month_end"
        ><UCheckbox
          :model-value="state.is_month_end"
          label="月末（その月の最終日）にする"
          @update:model-value="setRecurringMonthEnd(state, $event === true)"
      /></UFormField>
      <UFormField name="from_payment_method_id" label="移動元" required
        ><USelect
          v-model="state.from_payment_method_id"
          :items="paymentOptions"
          placeholder="選択してください"
          class="w-full"
      /></UFormField>
      <UFormField name="to_payment_method_id" label="移動先" required
        ><USelect
          v-model="state.to_payment_method_id"
          :items="paymentOptions"
          placeholder="選択してください"
          class="w-full"
      /></UFormField>
      <UFormField name="start_date" label="開始日" required
        ><DatePicker v-model="state.start_date" required class="w-full"
      /></UFormField>
      <UFormField name="end_date" label="終了日（任意）"
        ><DatePicker v-model="state.end_date" :min="state.start_date" clearable class="w-full"
      /></UFormField>
      <UFormField name="description" label="備考（任意）"
        ><UTextarea v-model="state.description" class="w-full"
      /></UFormField>
      <UFormField name="is_active"
        ><UCheckbox v-model="state.is_active" label="有効にする"
      /></UFormField>
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
