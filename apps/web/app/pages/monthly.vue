<script setup lang="ts">
import type { ExchangeRatePreview } from "~/types/transactions";
import {
  activeInMonth,
  budgetActuals,
  categoryTotals,
  inPeriod,
  mergeTransactions,
  recurringForecast,
  recurringPostedAmounts,
  sumAmounts,
  investmentAmount,
  type Transaction,
} from "~/utils/summaries";
import { groupTransactionsByDate } from "~/utils/transactions";

useSeoMeta({ title: "月間集計" });

const { month, setMonth } = useMonthQuery("/monthly");

const monthLabel = computed(() =>
  new Intl.DateTimeFormat("ja-JP", { year: "numeric", month: "long" }).format(
    new Date(`${month.value}-01T00:00:00`),
  ),
);

const {
  expenses,
  incomes,
  transfers,
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
const monthTransfers = computed(() => inPeriod(transfers.value, month.value));
const investment = computed(() =>
  investmentAmount(transfers.value, paymentMethods.value, month.value),
);

const usdPreviewsByMonth = useUsdRecurringPreviews(() => [month.value], recurringExpenses);
const usdRecurringPreviews = computed(
  () => usdPreviewsByMonth.value.get(month.value) ?? new Map<string, ExchangeRatePreview>(),
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
const transactions = computed(() =>
  mergeTransactions(monthExpenses.value, monthIncomes.value, monthTransfers.value),
);
const transactionGroups = computed(() => groupTransactionsByDate(transactions.value));
const postedRecurringAmounts = computed(() =>
  recurringPostedAmounts(monthExpenses.value, monthIncomes.value, month.value),
);

const activeRecurringExpenses = computed(() =>
  recurringExpenses.value.filter((item) => item.is_active && activeInMonth(item, month.value)),
);
const activeRecurringIncomes = computed(() =>
  recurringIncomes.value.filter((item) => item.is_active && activeInMonth(item, month.value)),
);

const missingVariableIncomes = computed(() =>
  activeRecurringIncomes.value.filter(
    (item) => item.is_variable && !postedRecurringAmounts.value.incomes.has(item.id),
  ),
);
const missingVariableItems = computed(() => [
  ...activeRecurringExpenses.value.filter(
    (item) => item.is_variable && !postedRecurringAmounts.value.expenses.has(item.id),
  ),
  ...missingVariableIncomes.value,
]);
function scrollToRecurring() {
  const first = missingVariableItems.value[0];
  const button = first
    ? document.querySelector<HTMLElement>(`[aria-label="${CSS.escape(first.name)}の今月分を登録"]`)
    : null;
  if (button) {
    button.scrollIntoView({ block: "center" });
    button.focus({ preventScroll: true });
    return;
  }
  document.getElementById("recurring-items")?.scrollIntoView({ block: "start" });
}

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
  if (item.kind === "transfer")
    return (
      item.description ||
      `${paymentNames.value.get(item.from_payment_method_id) ?? "不明"} → ${paymentNames.value.get(item.to_payment_method_id) ?? "不明"}`
    );
  return (
    item.description ||
    (item.kind === "expense"
      ? expenseNames.value.get(item.category_id)
      : incomeNames.value.get(item.category_id)) ||
    "名称なし"
  );
}

function transactionMeta(item: Transaction) {
  if (item.kind === "transfer")
    return `${paymentNames.value.get(item.from_payment_method_id) ?? "不明"} → ${paymentNames.value.get(item.to_payment_method_id) ?? "不明"}`;
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
  else if (item.kind === "income") editIncome(item);
  else editTransfer(item);
}

function deleteTransaction(item: Transaction) {
  if (item.kind === "expense") askDeleteExpense(item);
  else if (item.kind === "income") askDeleteIncome(item);
  else askDeleteTransfer(item);
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
  askDeleteTransfer,
  confirmDelete,
} = useTransactionEditor({ month, expenses, incomes, transfers });
const {
  transferFormOpen,
  editingTransfer,
  transferSaving,
  transferInitialDate,
  openNewTransfer,
  editTransfer,
  closeTransferForm,
  saveTransfer,
} = useTransferEditor({ month, transfers });
</script>

<template>
  <div class="flex min-w-0 flex-1">
    <UDashboardPanel class="pb-[calc(8rem+env(safe-area-inset-bottom))] sm:pb-0" id="monthly">
      <template #header>
        <UDashboardNavbar
          title="月間集計"
          :toggle="false"
          :ui="{
            title: 'sm:sr-only',
            root: 'sm:h-auto sm:min-h-(--ui-header-height) sm:flex-wrap sm:py-2',
          }"
        >
          <template #leading>
            <AppHeaderNav />
          </template>
          <template #right>
            <MonthNavigator :model-value="month" @update:model-value="setMonth" />
            <TransferButton
              class="hidden sm:inline-flex"
              :disabled="loading || Boolean(loadError)"
              @click="openNewTransfer"
            />
            <RecordTransactionButton
              class="hidden sm:inline-flex"
              :disabled="loading || Boolean(loadError)"
              @click="openNew"
            />
            <UColorModeButton class="hidden sm:inline-flex" />
            <AppHeaderActions />
          </template>
        </UDashboardNavbar>
      </template>
      <template #body>
        <div
          class="fixed right-4 bottom-[calc(4.25rem+env(safe-area-inset-bottom))] z-20 flex justify-end gap-3 sm:hidden [&>button]:shadow-lg"
        >
          <TransferButton :disabled="loading || Boolean(loadError)" @click="openNewTransfer" />
          <RecordTransactionButton :disabled="loading || Boolean(loadError)" @click="openNew" />
        </div>
        <DataLoadState :loading="loading" :error="loadError" @retry="load">
          <div class="space-y-6">
            <HomeSummaryPanel
              :month-label="monthLabel"
              :income="projectedIncomeTotal"
              :expense="projectedExpenseTotal"
              :forecast="forecast"
              :budget-total="budgetTotal"
              :budget-rate="budgetRate"
              :investment="investment"
              :missing-income-names="missingVariableIncomes.map((item) => item.name)"
            />

            <div
              v-if="missingVariableItems.length"
              class="flex min-w-0 items-center gap-2 rounded-lg border border-default bg-elevated/50 px-3 py-2 text-sm"
            >
              <UIcon name="i-lucide-calendar-clock" class="size-4 shrink-0 text-muted" />
              <span class="shrink-0">今月の未登録 {{ missingVariableItems.length }}件</span>
              <span class="min-w-0 flex-1 truncate text-muted">{{
                missingVariableItems.map((item) => item.name).join("・")
              }}</span>
              <UButton
                color="neutral"
                variant="outline"
                class="shrink-0 max-sm:min-h-[44px]"
                @click="scrollToRecurring"
                >登録する</UButton
              >
            </div>

            <div class="grid gap-6 lg:grid-cols-[1fr_1.6fr]">
              <div class="order-2 space-y-6 lg:order-1">
                <HomeSpendingBreakdown :spending="spending" :total="expenseTotal" />
                <HomeBudgetProgress :actuals="actuals" />
                <HomeRecurringItems
                  :recurring-expenses="activeRecurringExpenses"
                  :recurring-incomes="activeRecurringIncomes"
                  :usd-previews="usdRecurringPreviews"
                  :posted-expense-amounts="postedRecurringAmounts.expenses"
                  :posted-income-amounts="postedRecurringAmounts.incomes"
                  @register-expense="registerRecurringExpense"
                  @register-income="registerRecurringIncome"
                />
              </div>

              <HomeTransactionList
                class="order-1 lg:order-2"
                :month-label="monthLabel"
                :groups="transactionGroups"
                :count="transactions.length"
                :label-of="transactionLabel"
                :meta-of="transactionMeta"
                @create="openNew"
                @edit="editTransaction"
                @delete="deleteTransaction"
              />
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

    <TransactionsTransferFormModal
      v-if="transferFormOpen"
      v-model:open="transferFormOpen"
      :payment-methods="paymentMethods"
      :saving="transferSaving"
      :initial-date="transferInitialDate"
      :initial-transfer="editingTransfer"
      :on-submit="saveTransfer"
      @update:open="
        (value) => {
          if (!value) closeTransferForm();
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
