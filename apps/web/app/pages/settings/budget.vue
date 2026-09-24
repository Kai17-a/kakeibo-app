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
const items = ref<Budget[]>([]);
const categories = ref<Category[]>([]);
const deleting = ref<Budget | null>(null);
const state = reactive<{ category_id: string; amount: string }>({ category_id: "", amount: "" });
const sortedItems = computed(() => items.value);
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
  editingId.value = item?.id ?? null;
  Object.assign(state, { category_id: item?.category_id ?? "", amount: item?.amount ?? "" });
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
    const [budgets, expenseCategories] = await Promise.all([
      api.list(),
      settingsApi.categories("expense").list(),
    ]);
    items.value = budgets;
    categories.value = groupCategories(expenseCategories);
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
    const input = { category_id: state.category_id, amount: String(state.amount).trim() };
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
function askDelete(item: Budget) {
  deleting.value = item;
  deleteError.value = "";
  deleteOpen.value = true;
}
function budgetActions(item: Budget) {
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
  } catch {
    deleteError.value = "予算を削除できませんでした。";
  } finally {
    removing.value = false;
  }
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="budget-heading">
    <div class="flex min-w-0 items-start justify-between gap-3 sm:gap-4">
      <div class="min-w-0 space-y-1">
        <h2
          id="budget-heading"
          class="flex flex-wrap items-center gap-2 text-xl font-semibold text-highlighted"
        >
          {{ label }}
          <UBadge v-if="!loading && !loadError" color="neutral" variant="soft">
            {{ items.length }}件
          </UBadge>
        </h2>
        <p class="text-sm text-muted">支出カテゴリごとに毎月適用する予算を管理します。</p>
      </div>
      <UButton
        icon="i-lucide-plus"
        class="shrink-0"
        :disabled="loading || !!loadError || !unassignedCategories.length"
        @click="openForm()"
      >
        追加
      </UButton>
    </div>
    <p v-if="!loading && !loadError && !unassignedCategories.length" class="text-sm text-muted">
      予算未設定の支出カテゴリがありません。カテゴリを追加するか、登録済みの予算を編集してください。
    </p>
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
        <UIcon name="i-lucide-chart-pie" class="size-8 text-muted" />
        <h3 class="font-semibold">{{ label }}がありません</h3>
        <p class="text-sm text-muted">
          カテゴリごとの月額予算を設定すると、ここに一覧が表示されます。
        </p>
        <UButton
          color="neutral"
          variant="outline"
          icon="i-lucide-plus"
          :disabled="!unassignedCategories.length"
          @click="openForm()"
        >
          予算を追加
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
        <template #category_id-cell="{ row }">
          <span class="font-medium break-words text-default">{{
            categoryName(row.original.category_id)
          }}</span>
        </template>
        <template #actions-cell="{ row }">
          <UDropdownMenu :items="budgetActions(row.original)">
            <UButton
              icon="i-lucide-ellipsis"
              color="neutral"
              variant="ghost"
              size="sm"
              :aria-label="`${categoryName(row.original.category_id)}の操作`"
            />
          </UDropdownMenu>
        </template>
      </UTable>
    </div>
    <UModal
      v-model:open="formOpen"
      :title="`${label}を${editingId ? '編集' : '追加'}`"
      description="支出カテゴリと毎月の予算額を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="budget-form"
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
        </UForm>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" :disabled="saving" @click="formOpen = false">
          キャンセル
        </UButton>
        <UButton type="submit" form="budget-form" :loading="saving" :disabled="saving">
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
          {{ deleting ? categoryName(deleting.category_id) : "" }}
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
