<script setup lang="ts">
import SortableJs from "sortablejs";
import type { PaymentMethod } from "~/types/settings";
import { validateNamed, validAmount } from "~/utils/settings";
import { formatSignedCurrency } from "~/utils/format";

useSeoMeta({ title: "支払方法設定" });
const label = "支払方法";
const api = useSettingsApi().paymentMethods;
const state = reactive<{
  name: string;
  description: string;
  initial_balance: string;
  is_investment: boolean;
}>({
  name: "",
  description: "",
  initial_balance: "",
  is_investment: false,
});

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
const toast = useToast();
const reordering = ref(false);
const orderError = ref("");
async function move(item: PaymentMethod, direction: -1 | 1) {
  if (reordering.value) return;
  const ids = items.value.map((entry) => entry.id);
  const index = ids.indexOf(item.id);
  const target = index + direction;
  if (target < 0 || target >= ids.length) return;
  ids.splice(index, 1);
  ids.splice(target, 0, item.id);
  await reorder(ids);
}
async function reorder(ids: string[]) {
  if (reordering.value) return;
  const previous = items.value;
  items.value = ids.map((id, display_order) => ({
    ...previous.find((item) => item.id === id)!,
    display_order,
  }));
  reordering.value = true;
  orderError.value = "";
  try {
    await api.reorder(ids);
    toast.add({ title: "支払方法の表示順を更新しました", color: "success" });
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
      handle: ".payment-method-handle",
      draggable: ".payment-method-row",
      disabled: reordering.value,
      animation: 150,
      delay: 150,
      delayOnTouchOnly: true,
      touchStartThreshold: 5,
      onEnd: (event) => {
        const container = event.from;
        const ids = Array.from(container.children).map((el) => (el as HTMLElement).dataset.id!);
        const children = Array.from(container.children).filter((el) => el !== event.item);
        container.insertBefore(event.item, children[event.oldIndex ?? 0] ?? null);
        if (event.oldIndex !== event.newIndex) void reorder(ids);
      },
    });
  },
  { flush: "post" },
);
onBeforeUnmount(() => sortable?.destroy());

function validate() {
  const errors = validateNamed(state);
  if (String(state.initial_balance).trim() && !validAmount(state.initial_balance))
    errors.push({ name: "initial_balance", message: "0以上の整数で入力してください" });
  return errors;
}
function openForm(item?: PaymentMethod) {
  if (reordering.value) return;
  Object.assign(state, {
    name: item?.name ?? "",
    description: item?.description ?? "",
    initial_balance: item?.initial_balance ?? "",
    is_investment: item?.is_investment ?? false,
  });
  openCrudForm(item);
}
function save() {
  return saveItem({
    name: state.name.trim(),
    description: state.description.trim() || null,
    initial_balance: String(state.initial_balance).trim() || undefined,
    is_investment: state.is_investment,
  });
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="payment-method-heading">
    <SettingsListHeader
      heading-id="payment-method-heading"
      :title="label"
      description="支出の登録時に選択する支払方法と残高を管理します"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError || reordering"
      @add="openForm()"
    />

    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-wallet"
      empty-description="利用する支払方法と開始時点の残高を登録すると、ここに一覧が表示されます"
      add-label="支払方法を追加"
      @retry="load"
      @add="openForm()"
    >
      <div
        ref="list"
        class="divide-y divide-default overflow-hidden border-y border-default"
        :aria-label="`${label}一覧`"
      >
        <div
          v-for="item in items"
          :key="item.id"
          class="payment-method-row flex cursor-pointer items-center gap-2 px-3 py-2 text-sm hover:bg-elevated/50"
          :data-id="item.id"
          @click="openForm(item)"
        >
          <button
            type="button"
            class="payment-method-handle flex size-7 shrink-0 touch-none items-center justify-center rounded focus-visible:ring-2 focus-visible:ring-primary max-sm:size-[44px]"
            :aria-label="`${item.name}の並べ替え`"
            :disabled="reordering"
            @click.stop
            @keydown.up.prevent.stop="move(item, -1)"
            @keydown.down.prevent.stop="move(item, 1)"
          >
            <UIcon name="i-lucide-grip-vertical" class="size-5 text-muted" />
          </button>
          <div class="min-w-0 flex-1">
            <button
              type="button"
              class="text-left font-medium break-words text-default focus-visible:ring-2 focus-visible:ring-primary max-sm:min-h-[44px]"
              :aria-label="`${item.name}を編集`"
              @click.stop="openForm(item)"
            >
              {{ item.name }}
            </button>
            <p v-if="item.description" class="text-xs break-words whitespace-pre-wrap text-muted">
              {{ item.description }}
            </p>
          </div>
          <div class="w-28 shrink-0 text-right tabular-nums">
            <span class="mr-1.5 text-xs text-muted">残高</span>
            <span v-if="item.balance === null" class="text-muted">未設定</span>
            <span
              v-else
              :class="
                Number(item.balance) > 0
                  ? 'text-primary'
                  : Number(item.balance) < 0
                    ? 'text-error'
                    : 'text-default'
              "
            >
              {{ formatSignedCurrency(item.balance) }}
            </span>
          </div>
          <div @click.stop>
            <RowActionsMenu
              :label="item.name"
              @edit="openForm(item)"
              @delete="!reordering && askDelete(item)"
            />
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
      description="支払方法名、初期残高、説明を設定します"
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
        description="残高管理を行う場合の開始時点の金額を入力してください"
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
      <UFormField name="is_investment">
        <UCheckbox
          v-model="state.is_investment"
          label="投資口座として扱う"
          description="この口座への振替を投資額として集計します。変更すると過去の集計にも反映されます。"
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
