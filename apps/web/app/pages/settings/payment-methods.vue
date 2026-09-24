<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type { PaymentMethod } from "~/types/settings";
import { validateNamed, validAmount } from "~/utils/settings";
import { formatSignedCurrency } from "~/utils/format";

useSeoMeta({ title: "支払方法設定" });
const label = "支払方法";
const api = useSettingsApi().paymentMethods;
const state = reactive<{ name: string; description: string; initial_balance: string }>({
  name: "",
  description: "",
  initial_balance: "",
});
const columns: TableColumn<PaymentMethod>[] = [
  {
    accessorKey: "name",
    header: "支払方法名",
    meta: { class: { th: "w-full min-w-0", td: "w-full min-w-0" } },
  },
  {
    accessorKey: "balance",
    header: "残高",
    meta: {
      class: {
        th: "w-24 whitespace-nowrap text-right",
        td: "w-24 whitespace-nowrap text-right tabular-nums",
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
    "支払方法を削除できませんでした。登録済みの明細で使用されている場合は削除できません。",
});
const sortedItems = computed(() =>
  items.value.toSorted((a, b) => a.name.localeCompare(b.name, "ja")),
);

function validate() {
  const errors = validateNamed(state);
  if (String(state.initial_balance).trim() && !validAmount(state.initial_balance))
    errors.push({ name: "initial_balance", message: "0以上の整数で入力してください。" });
  return errors;
}
function openForm(item?: PaymentMethod) {
  Object.assign(state, {
    name: item?.name ?? "",
    description: item?.description ?? "",
    initial_balance: item?.initial_balance ?? "",
  });
  openCrudForm(item);
}
function save() {
  return saveItem({
    name: state.name.trim(),
    description: state.description.trim() || null,
    initial_balance: String(state.initial_balance).trim() || undefined,
  });
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="payment-method-heading">
    <SettingsListHeader
      heading-id="payment-method-heading"
      :title="label"
      description="支出の登録時に選択する支払方法と残高を管理します。"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError"
      @add="openForm()"
    />

    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-wallet"
      empty-description="利用する支払方法と開始時点の残高を登録すると、ここに一覧が表示されます。"
      add-label="支払方法を追加"
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
              <p class="font-medium break-words whitespace-normal text-default">
                {{ row.original.name }}
              </p>
              <p
                v-if="row.original.description"
                class="mt-0.5 text-xs break-words whitespace-pre-wrap text-muted"
              >
                {{ row.original.description }}
              </p>
            </div>
          </template>
          <template #balance-cell="{ row }">
            <span v-if="row.original.balance === null" class="text-muted">未設定</span>
            <span
              v-else
              :class="
                Number(row.original.balance) > 0
                  ? 'text-primary'
                  : Number(row.original.balance) < 0
                    ? 'text-error'
                    : 'text-default'
              "
            >
              {{ formatSignedCurrency(row.original.balance) }}
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
      description="支払方法名、初期残高、説明を設定します。"
      form-id="payment-method-form"
      :state="state"
      :validate="validate"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UFormField
        name="initial_balance"
        label="初期残高（任意）"
        description="残高管理を行う場合の開始時点の金額を入力してください。"
      >
        <UInput
          :model-value="state.initial_balance"
          type="number"
          :min="0"
          :step="1"
          class="w-full"
          @update:model-value="state.initial_balance = String($event ?? '')"
        />
      </UFormField>
      <UFormField name="name" label="支払方法名" required>
        <UInput v-model="state.name" :maxlength="100" class="w-full" autofocus />
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
