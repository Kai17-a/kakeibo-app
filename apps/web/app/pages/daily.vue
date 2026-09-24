<script setup lang="ts">
import type { Category, PaymentMethod, RecurringExpense } from "~/types/settings";
import type { Expense, Income } from "~/types/transactions";
import { categoryTotals, dailyCategoryTotals, inPeriod, sumAmounts } from "~/utils/summaries";
import { filterExpenses, filterIncomes } from "~/utils/filters";
import {
  currentMonth,
  formatCurrency,
  formatDate,
  formatSignedCurrency,
  isValidMonth,
  shiftMonth,
} from "~/utils/format";

useSeoMeta({ title: "日別集計" });

const route = useRoute();
const month = computed(() =>
  typeof route.query.month === "string" && isValidMonth(route.query.month)
    ? route.query.month
    : currentMonth(),
);
function setMonth(next: string) {
  navigateTo({ path: "/daily", query: { month: next } }, { replace: true });
}
const settingsApi = useSettingsApi();
const transactionsApi = useTransactionsApi();

const expenses = ref<Expense[]>([]);
const incomes = ref<Income[]>([]);
const expenseCategories = ref<Category[]>([]);
const incomeCategories = ref<Category[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const recurringExpenses = ref<RecurringExpense[]>([]);
const loading = ref(true);
const loadError = ref("");

async function loadAll() {
  loading.value = true;
  loadError.value = "";
  try {
    const [
      expenseData,
      incomeData,
      expenseCategoryData,
      incomeCategoryData,
      paymentData,
      recurringData,
    ] = await Promise.all([
      transactionsApi.expenses(),
      transactionsApi.incomes(),
      settingsApi.categories("expense").list(),
      settingsApi.categories("income").list(),
      settingsApi.paymentMethods.list(),
      settingsApi.recurringExpenses.list(),
    ]);
    expenses.value = expenseData;
    incomes.value = incomeData;
    expenseCategories.value = expenseCategoryData;
    incomeCategories.value = incomeCategoryData;
    paymentMethods.value = paymentData;
    recurringExpenses.value = recurringData;
  } catch (error) {
    loadError.value = apiErrorMessage(error);
  } finally {
    loading.value = false;
  }
}
onMounted(loadAll);

const monthExpenses = computed(() => inPeriod(expenses.value, month.value));
const monthIncomes = computed(() => inPeriod(incomes.value, month.value));

const tab = ref<"summary" | "details" | "income-details" | "categories">("summary");
const tabItems = [
  { label: "収支・明細", value: "summary" },
  { label: "支出明細", value: "details" },
  { label: "収入明細", value: "income-details" },
  { label: "月ごとのカテゴリ別支出", value: "categories" },
];

const keyword = ref("");
const categoryId = ref("");
const paymentMethodId = ref("");
const incomeKeyword = ref("");
const incomeCategoryId = ref("");
const incomePaymentMethodId = ref("");
const ALL_FILTER_VALUE = "__ALL__";

function clearFilters() {
  keyword.value = "";
  categoryId.value = "";
  paymentMethodId.value = "";
}
function clearIncomeFilters() {
  incomeKeyword.value = "";
  incomeCategoryId.value = "";
  incomePaymentMethodId.value = "";
}

const incomeTotal = computed(() => sumAmounts(monthIncomes.value));
const expenseTotal = computed(() => sumAmounts(monthExpenses.value));
const paymentNames = computed(
  () => new Map(paymentMethods.value.map((item) => [item.id, item.name])),
);
const categoryNames = computed(
  () => new Map(expenseCategories.value.map((item) => [item.id, item.name])),
);
const incomeCategoryNames = computed(
  () => new Map(incomeCategories.value.map((item) => [item.id, item.name])),
);

const incomeBreakdown = computed(() =>
  incomeCategories.value.map((category) => ({
    ...category,
    total: sumAmounts(monthIncomes.value.filter((item) => item.category_id === category.id)),
  })),
);
const paymentBreakdown = computed(() =>
  paymentMethods.value.map((method) => ({
    ...method,
    total: sumAmounts(monthExpenses.value.filter((item) => item.payment_method_id === method.id)),
  })),
);

const recurring = computed(() =>
  recurringExpenses.value.filter(
    (item) =>
      item.is_active &&
      !item.is_variable &&
      item.start_date.slice(0, 7) <= month.value &&
      (!item.end_date || item.end_date.slice(0, 7) >= month.value),
  ),
);
const fixedRecurringExpenseIds = computed(() => new Set(recurring.value.map((item) => item.id)));
const variableRecurringCategoryIds = computed(
  () =>
    new Set(
      recurringExpenses.value
        .filter(
          (item) =>
            item.is_active &&
            item.is_variable === true &&
            item.start_date.slice(0, 7) <= month.value &&
            (!item.end_date || item.end_date.slice(0, 7) >= month.value),
        )
        .map((item) => item.category_id),
    ),
);
function isLinkedToFixedRecurring(item: Expense) {
  return Boolean(
    item.recurring_expense_id && fixedRecurringExpenseIds.value.has(item.recurring_expense_id),
  );
}
const fixedVariable = computed(() =>
  monthExpenses.value.filter(
    (item) =>
      !isLinkedToFixedRecurring(item) && variableRecurringCategoryIds.value.has(item.category_id),
  ),
);
const variable = computed(() =>
  monthExpenses.value.filter(
    (item) =>
      !isLinkedToFixedRecurring(item) && !variableRecurringCategoryIds.value.has(item.category_id),
  ),
);
const recurringTotal = computed(() => sumAmounts(recurring.value));
const fixedVariableTotal = computed(() => sumAmounts(fixedVariable.value));
const fixedVariableTotals = computed(() =>
  categoryTotals(fixedVariable.value, expenseCategories.value).filter((item) => item.total),
);
const variableTotal = computed(() => sumAmounts(variable.value));
const variableTotals = computed(() =>
  categoryTotals(variable.value, expenseCategories.value).filter((item) => item.total),
);

const ledger = computed(() =>
  filterExpenses(monthExpenses.value, {
    keyword: keyword.value,
    categoryId: categoryId.value,
    paymentMethodId: paymentMethodId.value,
  }).toSorted((a, b) => a.transaction_date.localeCompare(b.transaction_date)),
);
const incomeLedger = computed(() =>
  filterIncomes(monthIncomes.value, {
    keyword: incomeKeyword.value,
    categoryId: incomeCategoryId.value,
    paymentMethodId: incomePaymentMethodId.value,
  }).toSorted((a, b) => a.transaction_date.localeCompare(b.transaction_date)),
);

function expenseLabel(item: Expense) {
  return item.description || categoryNames.value.get(item.category_id) || "支出";
}
function incomeLabel(item: Income) {
  return item.description || incomeCategoryNames.value.get(item.category_id) || "収入";
}
function expenseActions(item: Expense) {
  return [
    [
      { label: "編集", icon: "i-lucide-pencil", onSelect: () => editExpense(item) },
      {
        label: "削除",
        icon: "i-lucide-trash-2",
        color: "error" as const,
        onSelect: () => askDeleteExpense(item),
      },
    ],
  ];
}
function incomeActions(item: Income) {
  return [
    [
      { label: "編集", icon: "i-lucide-pencil", onSelect: () => editIncome(item) },
      {
        label: "削除",
        icon: "i-lucide-trash-2",
        color: "error" as const,
        onSelect: () => askDeleteIncome(item),
      },
    ],
  ];
}

const dailyRows = computed(() => dailyCategoryTotals(monthExpenses.value, month.value));
const activeExpenseCategories = computed(() =>
  expenseCategories.value.filter((category) =>
    monthExpenses.value.some((item) => item.category_id === category.id),
  ),
);

const {
  formOpen,
  editingExpense,
  editingIncome,
  exchangePreview,
  saving,
  initialDate,
  openNew,
  editExpense,
  editIncome,
  closeForm,
  save,
  deleteOpen,
  deleteDescription,
  removing,
  askDeleteExpense,
  askDeleteIncome,
  confirmDelete,
} = useTransactionEditor({ month, expenses, incomes });
</script>

<template>
  <div class="flex min-w-0 flex-1">
    <UDashboardPanel id="daily">
      <template #header>
        <UDashboardNavbar title="日別集計">
          <template #leading>
            <UDashboardSidebarCollapse />
          </template>
          <template #right>
            <UFieldGroup class="shrink-0">
              <UButton
                icon="i-lucide-chevron-left"
                color="neutral"
                variant="outline"
                aria-label="前月"
                @click="setMonth(shiftMonth(month, -1))"
              />
              <MonthPicker
                :model-value="month"
                class="w-24 sm:w-40"
                @update:model-value="setMonth"
              />
              <UButton
                icon="i-lucide-chevron-right"
                color="neutral"
                variant="outline"
                aria-label="翌月"
                @click="setMonth(shiftMonth(month, 1))"
              />
            </UFieldGroup>
            <UButton icon="i-lucide-plus" aria-label="記録する" @click="openNew">
              <span class="hidden sm:inline">記録する</span>
            </UButton>
          </template>
        </UDashboardNavbar>
      </template>
      <template #body>
        <UAlert
          v-if="loadError"
          color="error"
          title="データを読み込めませんでした"
          :description="loadError"
          :actions="[{ label: '再試行', color: 'error', variant: 'outline', onClick: loadAll }]"
          class="mb-6"
        />
        <div v-if="loading" class="space-y-4">
          <USkeleton class="h-28 w-full" />
          <USkeleton class="h-28 w-full" />
        </div>
        <UCard v-else class="shrink-0">
          <UTabs
            v-model="tab"
            orientation="horizontal"
            variant="pill"
            :content="false"
            :items="tabItems"
            :ui="{
              list: 'overflow-x-auto',
              trigger: 'shrink-0',
              label: 'overflow-visible text-clip whitespace-nowrap',
            }"
            class="mb-5 w-full"
          />

          <div v-if="tab === 'categories'">
            <h2 class="mb-3 text-base font-semibold">日ごとのカテゴリ別支出</h2>
            <div v-if="activeExpenseCategories.length" class="overflow-x-auto">
              <table class="w-full min-w-[760px] border-collapse text-sm">
                <thead class="bg-elevated">
                  <tr>
                    <th
                      class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
                    >
                      日付
                    </th>
                    <th
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="min-w-24 px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      {{ category.name }}
                    </th>
                    <th
                      class="sticky right-0 bg-elevated px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      合計
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="day in dailyRows" :key="day.date" class="border-t border-default">
                    <th
                      class="sticky left-0 w-32 min-w-32 bg-default px-3 py-2 text-left whitespace-nowrap"
                    >
                      {{ formatDate(day.date) }}
                    </th>
                    <td
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="px-3 py-2 text-right tabular-nums"
                    >
                      {{
                        day.values.get(category.id)
                          ? formatCurrency(day.values.get(category.id)!)
                          : ""
                      }}
                    </td>
                    <td
                      class="sticky right-0 bg-elevated px-3 py-2 text-right font-bold tabular-nums"
                    >
                      {{ day.total ? formatCurrency(day.total) : "" }}
                    </td>
                  </tr>
                </tbody>
                <tfoot class="border-t-2 border-default bg-elevated">
                  <tr>
                    <th
                      class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-3 text-left whitespace-nowrap"
                    >
                      合計
                    </th>
                    <td
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="px-3 py-3 text-right font-bold tabular-nums"
                    >
                      {{
                        formatCurrency(
                          sumAmounts(
                            monthExpenses.filter((item) => item.category_id === category.id),
                          ),
                        )
                      }}
                    </td>
                    <td
                      class="sticky right-0 bg-elevated px-3 py-3 text-right font-bold tabular-nums"
                    >
                      {{ formatCurrency(sumAmounts(monthExpenses)) }}
                    </td>
                  </tr>
                </tfoot>
              </table>
            </div>
            <p v-else class="py-10 text-center text-sm text-muted">
              この月の支出はまだありません。
            </p>
          </div>

          <div v-else-if="tab === 'details'" class="min-w-0">
            <h2 class="text-base font-semibold">支出明細</h2>
            <div class="flex flex-wrap items-end gap-3 border-b border-default py-4">
              <UFormField label="備考を検索" class="min-w-48 flex-1">
                <UInput
                  v-model="keyword"
                  type="search"
                  placeholder="キーワードを入力"
                  class="w-full"
                />
              </UFormField>
              <UFormField label="カテゴリ" class="min-w-40 flex-1 sm:flex-none">
                <USelect
                  :model-value="categoryId || ALL_FILTER_VALUE"
                  :items="[
                    { label: 'すべて', value: ALL_FILTER_VALUE },
                    ...expenseCategories.map((category) => ({
                      label: category.name,
                      value: category.id,
                    })),
                  ]"
                  class="w-full sm:w-44"
                  @update:model-value="
                    categoryId = $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)
                  "
                />
              </UFormField>
              <UFormField label="支払方法" class="min-w-40 flex-1 sm:flex-none">
                <USelect
                  :model-value="paymentMethodId || ALL_FILTER_VALUE"
                  :items="[
                    { label: 'すべて', value: ALL_FILTER_VALUE },
                    ...paymentMethods.map((method) => ({ label: method.name, value: method.id })),
                  ]"
                  class="w-full sm:w-44"
                  @update:model-value="
                    paymentMethodId =
                      $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)
                  "
                />
              </UFormField>
              <UButton color="neutral" variant="outline" @click="clearFilters">
                条件をクリア
              </UButton>
            </div>
            <div class="max-h-[38rem] overflow-auto">
              <table class="w-full min-w-[800px] text-sm">
                <thead class="sticky top-0 bg-default">
                  <tr class="border-b border-default">
                    <th
                      class="w-32 px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
                    >
                      日付
                    </th>
                    <th
                      class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      摘要
                    </th>
                    <th
                      class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      支払方法
                    </th>
                    <th
                      class="w-32 px-3 py-2 text-right text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
                    >
                      金額
                    </th>
                    <th
                      class="min-w-40 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      備考
                    </th>
                    <th class="w-36 px-3 py-2 text-right">
                      <span class="sr-only">操作</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="item in ledger"
                    :key="item.id"
                    class="border-t border-default transition-colors hover:bg-elevated/50"
                  >
                    <td class="px-3 py-2 whitespace-nowrap">
                      {{ formatDate(item.transaction_date) }}
                    </td>
                    <td class="px-3 py-2">
                      {{ categoryNames.get(item.category_id) }}
                    </td>
                    <td class="px-3 py-2">
                      {{ paymentNames.get(item.payment_method_id) }}
                    </td>
                    <td class="px-3 py-2 text-right whitespace-nowrap tabular-nums">
                      {{ formatSignedCurrency(item.amount, "negative") }}
                    </td>
                    <td class="px-3 py-2">
                      {{ item.description ?? "" }}
                    </td>
                    <td class="px-3 py-2 text-right">
                      <UDropdownMenu :items="expenseActions(item)">
                        <UButton
                          icon="i-lucide-ellipsis"
                          color="neutral"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${expenseLabel(item)}の操作`"
                        />
                      </UDropdownMenu>
                    </td>
                  </tr>
                </tbody>
              </table>
              <p v-if="!ledger.length" class="py-12 text-center text-sm text-muted">
                {{
                  monthExpenses.length
                    ? "条件に一致する明細がありません。"
                    : "この月の支出明細はありません。"
                }}
              </p>
            </div>
          </div>

          <div v-else-if="tab === 'income-details'" class="min-w-0">
            <h2 class="text-base font-semibold">収入明細</h2>
            <div class="flex flex-wrap items-end gap-3 border-b border-default py-4">
              <UFormField label="備考を検索" class="min-w-48 flex-1">
                <UInput
                  v-model="incomeKeyword"
                  type="search"
                  placeholder="キーワードを入力"
                  class="w-full"
                />
              </UFormField>
              <UFormField label="カテゴリ" class="min-w-40 flex-1 sm:flex-none">
                <USelect
                  :model-value="incomeCategoryId || ALL_FILTER_VALUE"
                  :items="[
                    { label: 'すべて', value: ALL_FILTER_VALUE },
                    ...incomeCategories.map((category) => ({
                      label: category.name,
                      value: category.id,
                    })),
                  ]"
                  class="w-full sm:w-44"
                  @update:model-value="
                    incomeCategoryId =
                      $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)
                  "
                />
              </UFormField>
              <UFormField label="支払方法" class="min-w-40 flex-1 sm:flex-none">
                <USelect
                  :model-value="incomePaymentMethodId || ALL_FILTER_VALUE"
                  :items="[
                    { label: 'すべて', value: ALL_FILTER_VALUE },
                    ...paymentMethods.map((method) => ({ label: method.name, value: method.id })),
                  ]"
                  class="w-full sm:w-44"
                  @update:model-value="
                    incomePaymentMethodId =
                      $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)
                  "
                />
              </UFormField>
              <UButton color="neutral" variant="outline" @click="clearIncomeFilters">
                条件をクリア
              </UButton>
            </div>
            <div class="max-h-[38rem] overflow-auto">
              <table class="w-full min-w-[800px] text-sm">
                <thead class="sticky top-0 bg-default">
                  <tr class="border-b border-default">
                    <th
                      class="w-32 px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
                    >
                      日付
                    </th>
                    <th
                      class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      摘要
                    </th>
                    <th
                      class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      支払方法
                    </th>
                    <th
                      class="w-32 px-3 py-2 text-right text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
                    >
                      金額
                    </th>
                    <th
                      class="min-w-40 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      備考
                    </th>
                    <th class="w-36 px-3 py-2 text-right">
                      <span class="sr-only">操作</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="item in incomeLedger"
                    :key="item.id"
                    class="border-t border-default transition-colors hover:bg-elevated/50"
                  >
                    <td class="px-3 py-2 whitespace-nowrap">
                      {{ formatDate(item.transaction_date) }}
                    </td>
                    <td class="px-3 py-2">
                      {{ incomeCategoryNames.get(item.category_id) }}
                    </td>
                    <td class="px-3 py-2">
                      {{
                        item.payment_method_id
                          ? (paymentNames.get(item.payment_method_id) ?? "—")
                          : "—"
                      }}
                    </td>
                    <td
                      :class="[
                        'px-3 py-2 text-right whitespace-nowrap tabular-nums',
                        Number(item.amount) > 0 ? 'text-primary' : 'text-default',
                      ]"
                    >
                      {{ formatSignedCurrency(item.amount, "positive") }}
                    </td>
                    <td class="px-3 py-2">
                      {{ item.description ?? "" }}
                    </td>
                    <td class="px-3 py-2 text-right">
                      <UDropdownMenu :items="incomeActions(item)">
                        <UButton
                          icon="i-lucide-ellipsis"
                          color="neutral"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${incomeLabel(item)}の操作`"
                        />
                      </UDropdownMenu>
                    </td>
                  </tr>
                </tbody>
              </table>
              <p v-if="!incomeLedger.length" class="py-12 text-center text-sm text-muted">
                {{
                  monthIncomes.length
                    ? "条件に一致する明細がありません。"
                    : "この月の収入明細はありません。"
                }}
              </p>
            </div>
          </div>

          <div v-else class="grid gap-8 lg:grid-cols-2">
            <aside>
              <h2 class="border-b border-default pb-2 text-base font-semibold">収支サマリー</h2>
              <dl class="grid grid-cols-2 text-sm">
                <dt class="p-2">収入</dt>
                <dd
                  :class="[
                    'p-2 text-right tabular-nums',
                    incomeTotal > 0 ? 'text-primary' : 'text-default',
                  ]"
                >
                  {{ formatSignedCurrency(incomeTotal, "positive") }}
                </dd>
                <dt class="p-2">支出</dt>
                <dd class="p-2 text-right tabular-nums">
                  {{ formatSignedCurrency(expenseTotal, "negative") }}
                </dd>
                <dt class="p-2 font-bold">収支</dt>
                <dd
                  :class="[
                    'p-2 text-right font-bold tabular-nums',
                    incomeTotal - expenseTotal < 0
                      ? 'text-error'
                      : incomeTotal - expenseTotal > 0
                        ? 'text-primary'
                        : 'text-default',
                  ]"
                >
                  {{ formatSignedCurrency(incomeTotal - expenseTotal) }}
                </dd>
              </dl>
              <h2 class="mt-5 border-b border-default pb-2 text-base font-semibold">収入</h2>
              <div
                v-for="item in incomeBreakdown"
                :key="item.id"
                class="flex justify-between gap-4 px-2 py-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
              </div>
              <h2 class="mt-5 border-b border-default pb-2 text-base font-semibold">支払種別</h2>
              <div
                v-for="item in paymentBreakdown"
                :key="item.id"
                class="flex justify-between gap-4 px-2 py-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
              </div>
            </aside>
            <div>
              <div class="flex items-center justify-between gap-4 border-b border-default pb-2">
                <h2 class="text-sm font-bold">支出（固定費）</h2>
                <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
                  formatCurrency(recurringTotal)
                }}</span>
              </div>
              <div
                v-for="item in recurring"
                :key="item.id"
                class="flex items-center justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
              >
                <span>
                  {{ item.name }}
                  <small class="block text-muted">毎月{{ item.payment_day }}日</small>
                </span>
                <span class="text-right tabular-nums">{{ formatCurrency(item.amount) }}</span>
              </div>
              <div
                class="mt-5 flex items-center justify-between gap-4 border-b border-default pb-2"
              >
                <h2 class="text-sm font-bold">準固定費（金額変動）</h2>
                <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
                  formatCurrency(fixedVariableTotal)
                }}</span>
              </div>
              <div
                v-for="item in fixedVariableTotals"
                :key="item.id"
                class="flex justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
              </div>
              <div
                class="mt-5 flex items-center justify-between gap-4 border-b border-default pb-2"
              >
                <h2 class="text-sm font-bold">支出（変動費）</h2>
                <span class="text-sm font-bold whitespace-nowrap tabular-nums">{{
                  formatCurrency(variableTotal)
                }}</span>
              </div>
              <div
                v-for="item in variableTotals"
                :key="item.id"
                class="flex justify-between gap-4 border-b border-muted px-2 py-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span class="text-right tabular-nums">{{ formatCurrency(item.total) }}</span>
              </div>
            </div>
          </div>
        </UCard>
      </template>
    </UDashboardPanel>

    <TransactionsTransactionFormModal
      v-if="formOpen"
      v-model:open="formOpen"
      :expense-categories="expenseCategories"
      :income-categories="incomeCategories"
      :payment-methods="paymentMethods"
      :saving="saving"
      :initial-date="initialDate"
      :initial-expense="editingExpense"
      :initial-income="editingIncome"
      :exchange-preview="exchangePreview"
      :on-submit="save"
      @update:open="
        (value) => {
          if (!value) closeForm();
        }
      "
    />

    <ConfirmDeleteModal
      v-model:open="deleteOpen"
      :description="deleteDescription"
      :busy="removing"
      @confirm="confirmDelete"
    />
  </div>
</template>
