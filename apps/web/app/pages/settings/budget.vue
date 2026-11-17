<script setup lang="ts">
import type { Budget, Category } from "~/types/settings";
import { groupCategories, validAmount } from "~/utils/settings";
import { formatCurrency } from "~/utils/format";

useSeoMeta({ title: "予算設定" });
const settingsApi = useSettingsApi();
const api = settingsApi.budgets;
const toast = useToast();
const categories = ref<Category[]>([]);
const budgets = ref<Budget[]>([]);
const values = ref<Record<string, string>>({});
const loading = ref(true);
const saving = ref(false);
const loadError = ref("");
const saveError = ref("");
const original = (id: string) => budgets.value.find((b) => b.category_id === id);
const normalized = (value: string) =>
  value.trim() && validAmount(value) ? String(Number(value)) : value.trim();
const changed = computed(() =>
  categories.value.filter(
    (c) => normalized(values.value[c.id] ?? "") !== normalized(original(c.id)?.amount ?? ""),
  ),
);
const invalid = (id: string) => !!values.value[id]?.trim() && !validAmount(values.value[id]!);
const hasErrors = computed(() => categories.value.some((c) => invalid(c.id)));
const total = computed(() =>
  categories.value.reduce(
    (sum, c) => sum + (validAmount(values.value[c.id] ?? "") ? Number(values.value[c.id]) : 0),
    0,
  ),
);
function reset() {
  values.value = Object.fromEntries(
    categories.value.map((c) => [c.id, original(c.id)?.amount ?? ""]),
  );
  saveError.value = "";
}
async function load() {
  loading.value = true;
  loadError.value = "";
  try {
    const [rows, cats] = await Promise.all([api.list(), settingsApi.categories("expense").list()]);
    budgets.value = rows;
    categories.value = groupCategories(cats);
    reset();
  } catch (error) {
    loadError.value = apiErrorMessage(error);
  } finally {
    loading.value = false;
  }
}
async function save() {
  if (saving.value || hasErrors.value || !changed.value.length) return;
  saving.value = true;
  saveError.value = "";
  try {
    for (const category of changed.value) {
      const existing = original(category.id);
      const amount = normalized(values.value[category.id] ?? "");
      if (!amount && existing) {
        await api.remove(existing.id);
        budgets.value = budgets.value.filter((b) => b.id !== existing.id);
      } else if (amount) {
        const input = { category_id: category.id, amount };
        const result = existing ? await api.update(existing.id, input) : await api.create(input);
        budgets.value = [...budgets.value.filter((b) => b.category_id !== category.id), result];
      }
    }
    reset();
    toast.add({ title: "予算を保存しました", color: "success" });
  } catch (error) {
    saveError.value = apiErrorMessage(error);
  } finally {
    saving.value = false;
  }
}
onMounted(load);
onBeforeRouteLeave(
  () => !changed.value.length || confirm("保存していない変更があります。移動しますか？"),
);
</script>

<template>
  <section class="space-y-6" aria-labelledby="budget-heading">
    <header>
      <h2 id="budget-heading" class="text-base font-semibold text-highlighted">予算</h2>
      <p class="mt-1 text-sm text-muted">
        支出カテゴリごとに毎月適用する予算を管理します。空欄は予算なしです。
      </p>
    </header>
    <p v-if="loading" role="status">読み込み中…</p>
    <div v-else-if="loadError" class="space-y-3">
      <UAlert color="error" :description="loadError" title="読み込めませんでした" />
      <UButton class="max-sm:min-h-[44px]" color="neutral" variant="outline" @click="load"
        >再試行</UButton
      >
    </div>
    <div v-else-if="!categories.length" class="space-y-3 border-y border-default p-6">
      <p>支出カテゴリがありません</p>
      <UButton to="/settings/expense-categories" class="max-sm:min-h-[44px]"
        >カテゴリ設定へ</UButton
      >
    </div>
    <template v-else>
      <div class="flex flex-wrap items-center justify-between gap-3">
        <p class="text-sm font-medium tabular-nums">月額予算の合計 {{ formatCurrency(total) }}</p>
        <div class="flex gap-2">
          <UButton
            color="neutral"
            variant="outline"
            class="max-sm:min-h-[44px]"
            :disabled="saving || !changed.length"
            @click="reset"
            >元に戻す</UButton
          ><UButton
            class="max-sm:min-h-[44px]"
            :loading="saving"
            :disabled="saving || !changed.length || hasErrors"
            @click="save"
            >変更を保存</UButton
          >
        </div>
      </div>
      <UAlert
        v-if="saveError"
        color="error"
        title="保存できませんでした"
        :description="saveError"
      />
      <div class="divide-y divide-default border-y border-default">
        <div
          v-for="category in categories"
          :key="category.id"
          class="flex items-center justify-between gap-3 px-3 py-2 text-sm sm:px-4"
        >
          <label
            :for="`budget-${category.id}`"
            :class="['min-w-0 break-words', category.parent_category_id ? 'pl-6' : '']"
            >{{ category.name }}</label
          >
          <div class="w-40 shrink-0">
            <UInput
              :id="`budget-${category.id}`"
              v-model="values[category.id]"
              inputmode="numeric"
              :aria-label="`${category.name}の月額予算`"
              :aria-invalid="invalid(category.id)"
              :aria-describedby="invalid(category.id) ? `error-${category.id}` : undefined"
              :disabled="saving"
              placeholder="予算なし"
              class="w-full"
              :ui="{ base: 'text-right tabular-nums max-sm:min-h-[44px] max-sm:text-[16px]' }"
              ><template #leading>￥</template></UInput
            >
            <p
              v-if="invalid(category.id)"
              :id="`error-${category.id}`"
              class="mt-1 text-xs text-error"
            >
              0以上の整数で入力してください
            </p>
          </div>
        </div>
      </div>
    </template>
  </section>
</template>
