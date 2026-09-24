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
const items = ref<RecurringIncome[]>([]);
const deleting = ref<RecurringIncome | null>(null);
const state = reactive(recurringIncomeForm());
const sortedItems = computed(() =>
  [...items.value].sort((a, b) => a.name.localeCompare(b.name, "ja")),
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
    meta: { class: { th: "hidden sm:table-cell", td: "hidden sm:table-cell" } },
  },
  {
    accessorKey: "payment_day",
    header: "入金日",
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
function validate() {
  return validateRecurringIncome(state, categories.value);
}

function openForm(item?: RecurringIncome) {
  editingId.value = item?.id ?? null;
  Object.assign(state, recurringIncomeForm(item));
  formError.value = "";
  formOpen.value = true;
}

const toast = useToast();
const loading = ref(true);
const loadError = ref("");
const formOpen = ref(false);
const deleteOpen = ref(false);
const editingId = ref<string | null>(null);
const saving = ref(false);
const removing = ref(false);
const formError = ref("");
const deleteError = ref("");

async function load() {
  loading.value = true;
  loadError.value = "";
  try {
    const [incomes, incomeCategories] = await Promise.all([
      api.list(),
      settingsApi.categories("income").list(),
    ]);
    items.value = incomes;
    categories.value = incomeCategories;
  } catch (error) {
    loadError.value = apiErrorMessage(error);
  } finally {
    loading.value = false;
  }
}
onMounted(load);

async function save() {
  if (saving.value) return;
  saving.value = true;
  formError.value = "";
  try {
    const input = recurringIncomeInput(state);
    const result = editingId.value
      ? await api.update(editingId.value, input)
      : await api.create(input);
    items.value = editingId.value
      ? items.value.map((item) => (item.id === result.id ? result : item))
      : [...items.value, result];
    formOpen.value = false;
    toast.add({
      title: `${label}を${editingId.value ? "更新" : "追加"}しました`,
      color: "success",
    });
  } catch (error) {
    formError.value = apiErrorMessage(error);
  } finally {
    saving.value = false;
  }
}
function askDelete(item: RecurringIncome) {
  deleting.value = item;
  deleteError.value = "";
  deleteOpen.value = true;
}
function recurringIncomeActions(item: RecurringIncome) {
  return [
    [
      { label: "編集", icon: "i-lucide-pencil", onSelect: () => openForm(item) },
      {
        label: "削除",
        icon: "i-lucide-trash-2",
        color: "error" as const,
        onSelect: () => askDelete(item),
      },
    ],
  ];
}
async function remove() {
  if (!deleting.value || removing.value) return;
  removing.value = true;
  deleteError.value = "";
  const id = deleting.value.id;
  try {
    await api.remove(id);
    items.value = items.value.filter((item) => item.id !== id);
    deleteOpen.value = false;
    toast.add({ title: `${label}を削除しました`, color: "success" });
  } catch (error) {
    deleteError.value = apiErrorMessage(error);
  } finally {
    removing.value = false;
  }
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="recurring-income-heading">
    <div class="flex min-w-0 items-start justify-between gap-3 sm:gap-4">
      <div class="min-w-0 space-y-1">
        <h2
          id="recurring-income-heading"
          class="flex flex-wrap items-center gap-2 text-xl font-semibold text-highlighted"
        >
          {{ label }}
          <UBadge v-if="!loading && !loadError" color="neutral" variant="soft">
            {{ items.length }}件
          </UBadge>
        </h2>
        <p class="text-sm text-muted">
          毎月発生する固定収入・準固定収入（金額変動）の入金予定を管理します。
        </p>
      </div>
      <UButton
        icon="i-lucide-plus"
        class="shrink-0"
        :disabled="loading || !!loadError"
        @click="openForm()"
      >
        追加
      </UButton>
    </div>

    <div v-if="loading" role="status" :aria-label="`${label}を読み込み中`" class="space-y-3">
      <USkeleton v-for="i in 3" :key="i" class="h-16 w-full" />
      <span class="sr-only">読み込み中…</span>
    </div>
    <UAlert
      v-else-if="loadError"
      color="error"
      :title="`${label}を取得できませんでした`"
      :description="loadError"
      :actions="[{ label: '再試行', color: 'error', variant: 'outline', onClick: load }]"
    />
    <UCard v-else-if="!items.length" class="text-center">
      <div class="space-y-3 py-10">
        <UIcon name="i-lucide-repeat" class="size-8 text-muted" />
        <h3 class="font-semibold">{{ label }}がありません</h3>
        <p class="text-sm text-muted">毎月の定期収入を登録すると、ここに一覧が表示されます。</p>
        <UButton color="neutral" variant="outline" icon="i-lucide-plus" @click="openForm()">
          定期収入を追加
        </UButton>
      </div>
    </UCard>
    <div v-else class="min-w-0 overflow-hidden rounded-lg border border-default">
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
        <template #actions-cell="{ row }">
          <UDropdownMenu :items="recurringIncomeActions(row.original)">
            <UButton
              icon="i-lucide-ellipsis"
              color="neutral"
              variant="ghost"
              size="sm"
              :aria-label="`${row.original.name}の操作`"
            />
          </UDropdownMenu>
        </template>
      </UTable>
    </div>
    <UModal
      v-model:open="formOpen"
      :title="`${label}を${editingId ? '編集' : '追加'}`"
      description="毎月の収入額、入金日、適用期間を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="recurring-income-form"
          :state="state"
          :validate="validate"
          :disabled="saving"
          class="space-y-5"
          @submit="save"
        >
          <UAlert
            v-if="formError"
            color="error"
            title="保存できませんでした"
            :description="formError"
          />
          <UAlert
            v-if="!categories.length"
            color="warning"
            title="収入カテゴリがありません"
            description="設定の「収入カテゴリ」でカテゴリを登録してから追加してください。"
          />
          <UFormField name="name" label="名称" required>
            <UInput v-model="state.name" class="w-full" autofocus />
          </UFormField>
          <UFormField name="amount" :label="state.is_variable ? '金額（目安）' : '金額'" required>
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
        </UForm>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" :disabled="saving" @click="formOpen = false">
          キャンセル
        </UButton>
        <UButton type="submit" form="recurring-income-form" :loading="saving" :disabled="saving">
          {{ editingId ? "更新" : "追加" }}
        </UButton>
      </template>
    </UModal>
    <UModal
      v-model:open="deleteOpen"
      :title="`${label}を削除`"
      description="この設定を削除します。この操作は取り消せません。"
      :dismissible="!removing"
      :close="!removing"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <p class="break-all">
          {{ deleting?.name }}
        </p>
        <UAlert
          v-if="deleteError"
          color="error"
          title="削除できませんでした"
          :description="deleteError"
          class="mt-4"
        />
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" :disabled="removing" @click="deleteOpen = false">
          キャンセル
        </UButton>
        <UButton color="error" :loading="removing" :disabled="removing" @click="remove">
          削除
        </UButton>
      </template>
    </UModal>
  </section>
</template>
