<script setup lang="ts">
import type { ExchangeRatePreview } from "~/types/transactions";
import {
  budgetActuals,
  categoryTotals,
  inPeriod,
  mergeTransactions,
  recurringForecast,
  sumAmounts,
  type Transaction,
} from "~/utils/summaries";
import { groupTransactionsByDate } from "~/utils/transactions";

useSeoMeta({ title: "月間集計" });

const { month, setMonth } = useMonthQuery("/");

const monthLabel = computed(() =>
  new Intl.DateTimeFormat("ja-JP", { year: "numeric", month: "long" }).format(
    new Date(`${month.value}-01T00:00:00`),
  ),
);

const transactionsApi = useTransactionsApi();
const {
  expenses,
  incomes,
  expenseCategories,
  incomeCategories,
  paymentMethods,
  recurringExpenses,
  recurringIncomes,
  budgets,
  loading,
  loadError,
  load,
} = useFinanceData({ incomeCategories: true, recurringIncomes: true, budgets: true });

const monthExpenses = computed(() => inPeriod(expenses.value, month.value));
const monthIncomes = computed(() => inPeriod(incomes.value, month.value));

const usdRecurringPreviews = ref(new Map<string, ExchangeRatePreview>());
let usdPreviewRequestId = 0;
watch(
  [month, recurringExpenses],
  async ([currentMonthValue]) => {
    const requestId = ++usdPreviewRequestId;
    const targets = recurringExpenses.value.filter(
      (item) => item.is_active && !item.is_variable && item.currency_code === "USD",
    );
    if (!targets.length) {
      if (requestId === usdPreviewRequestId) usdRecurringPreviews.value = new Map();
      return;
    }
    const results = await Promise.allSettled(
      targets.map(
        async (item) =>
          [
            item.id,
            await transactionsApi.previewRecurringExpenseExchangeRate(item.id, currentMonthValue),
          ] as const,
      ),
    );
    if (requestId !== usdPreviewRequestId) return;
    usdRecurringPreviews.value = new Map(
      results.flatMap((result) => (result.status === "fulfilled" ? [result.value] : [])),
    );
  },
  { immediate: true },
);

const expenseTotal = computed(() => sumAmounts(monthExpenses.value));
const incomeTotal = computed(() => sumAmounts(monthIncomes.value));
const forecast = computed(() =>
  recurringForecast(
    monthExpenses.value,
    monthIncomes.value,
    recurringExpenses.value,
    recurringIncomes.value,
    month.value,
    usdRecurringPreviews.value,
  ),
);
const projectedExpenseTotal = computed(() => expenseTotal.value + forecast.value.expense);
const projectedIncomeTotal = computed(() => incomeTotal.value + forecast.value.income);
const transactions = computed(() => mergeTransactions(monthExpenses.value, monthIncomes.value));
const transactionGroups = computed(() => groupTransactionsByDate(transactions.value));

const activeRecurringExpenses = computed(() =>
  recurringExpenses.value.filter((item) => item.is_active),
);
const activeRecurringIncomes = computed(() =>
  recurringIncomes.value.filter((item) => item.is_active),
);

const expenseNames = computed(
  () => new Map(expenseCategories.value.map((item) => [item.id, item.name])),
);
const incomeNames = computed(
  () => new Map(incomeCategories.value.map((item) => [item.id, item.name])),
);
const paymentNames = computed(
  () => new Map(paymentMethods.value.map((item) => [item.id, item.name])),
);

const spending = computed(() =>
  categoryTotals(monthExpenses.value, expenseCategories.value)
    .filter((item) => item.total)
    .toSorted((a, b) => b.total - a.total),
);
const actuals = computed(() =>
  budgetActuals(monthExpenses.value, expenseCategories.value, budgets.value, month.value),
);
const budgetTotal = computed(() => actuals.value.reduce((sum, item) => sum + item.budget, 0));
const budgetSpent = computed(() => actuals.value.reduce((sum, item) => sum + item.actual, 0));
const budgetRate = computed(() =>
  budgetTotal.value ? (budgetSpent.value / budgetTotal.value) * 100 : 0,
);
function transactionLabel(item: Transaction) {
  return (
    item.description ||
    (item.kind === "expense"
      ? expenseNames.value.get(item.category_id)
      : incomeNames.value.get(item.category_id)) ||
    "名称なし"
  );
}

function transactionMeta(item: Transaction) {
  const category =
    item.kind === "expense"
      ? expenseNames.value.get(item.category_id)
      : incomeNames.value.get(item.category_id);
  const payment = item.payment_method_id
    ? paymentNames.value.get(item.payment_method_id)
    : undefined;
  const label = transactionLabel(item);
  return [label === category ? undefined : category, payment].filter(Boolean).join(" · ");
}

function editTransaction(item: Transaction) {
  if (item.kind === "expense") editExpense(item);
  else editIncome(item);
}

function deleteTransaction(item: Transaction) {
  if (item.kind === "expense") askDeleteExpense(item);
  else askDeleteIncome(item);
}

const {
  formOpen,
  editingExpense,
  editingIncome,
  recurringExpensePreset,
  recurringIncomePreset,
  exchangePreview,
  saving,
  initialDate,
  openNew,
  editExpense,
  editIncome,
  registerRecurringExpense,
  registerRecurringIncome,
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
    <UDashboardPanel id="home">
      <template #header>
        <UDashboardNavbar title="ホーム">
          <template #leading>
            <UDashboardSidebarCollapse />
          </template>
          <template #right>
            <MonthNavigator :model-value="month" @update:model-value="setMonth" />
            <UButton icon="i-lucide-plus" aria-label="記録する" @click="openNew">
              <span class="hidden sm:inline">記録する</span>
            </UButton>
          </template>
        </UDashboardNavbar>
      </template>
      <template #body>
        <DataLoadState :loading="loading" :error="loadError" @retry="load">
          <div class="space-y-6">
            <HomeSummaryPanel
              :month-label="monthLabel"
              :income="projectedIncomeTotal"
              :expense="projectedExpenseTotal"
              :forecast="forecast"
              :budget-total="budgetTotal"
              :budget-rate="budgetRate"
            />

            <div class="grid gap-6 lg:grid-cols-[1.6fr_1fr]">
              <HomeTransactionList
                :month-label="monthLabel"
                :groups="transactionGroups"
                :count="transactions.length"
                :label-of="transactionLabel"
                :meta-of="transactionMeta"
                @create="openNew"
                @edit="editTransaction"
                @delete="deleteTransaction"
              />

              <div class="space-y-6">
                <HomeSpendingBreakdown :spending="spending" :total="expenseTotal" />
                <HomeBudgetProgress :actuals="actuals" />
                <HomeRecurringItems
                  :recurring-expenses="activeRecurringExpenses"
                  :recurring-incomes="activeRecurringIncomes"
                  :usd-previews="usdRecurringPreviews"
                  @register-expense="registerRecurringExpense"
                  @register-income="registerRecurringIncome"
                />
              </div>
            </div>
          </div>
        </DataLoadState>
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
      :initial-recurring="recurringExpensePreset"
      :initial-recurring-income="recurringIncomePreset"
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
