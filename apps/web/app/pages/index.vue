<script setup lang="ts">
import { annualCategoryRows, monthOfYear, projectedAnnualMonths } from "~/utils/annual";
import { categoryMonthlyTotals, paymentMethodBalanceTrend } from "~/utils/summaries";
import { currentMonth, currentYear } from "~/utils/format";

useSeoMeta({ title: "年間集計" });

const { year, setYear } = useYearQuery("/");

const {
  expenses,
  incomes,
  expenseCategories,
  incomeCategories,
  paymentMethods,
  recurringExpenses,
  recurringIncomes,
  loading,
  loadError,
  load,
} = useFinanceData({ incomeCategories: true, recurringIncomes: true });

const availableYears = computed(() => {
  const years = new Set(
    [...expenses.value, ...incomes.value].map((item) => item.transaction_date.slice(0, 4)),
  );
  years.add(year.value);
  return [...years].toSorted((a, b) => b.localeCompare(a));
});

const usdPreviewsByMonth = useUsdRecurringPreviews(
  () => Array.from({ length: 12 }, (_, index) => monthOfYear(year.value, index + 1)),
  recurringExpenses,
);

const projectedMonths = computed(() =>
  projectedAnnualMonths({
    year: year.value,
    expenses: expenses.value,
    incomes: incomes.value,
    recurringExpenses: recurringExpenses.value,
    recurringIncomes: recurringIncomes.value,
    usdPreviewsByMonth: usdPreviewsByMonth.value,
  }),
);
const expenseTotal = computed(() =>
  projectedMonths.value.reduce((sum, item) => sum + item.expense, 0),
);
const incomeTotal = computed(() =>
  projectedMonths.value.reduce((sum, item) => sum + item.income, 0),
);

const categoryRows = computed(() =>
  annualCategoryRows(year.value, expenses.value, expenseCategories.value, projectedMonths.value),
);
const spending = computed(() => categoryRows.value.toSorted((a, b) => b.total - a.total));

const categoryMonths = computed(() =>
  categoryMonthlyTotals(expenses.value, expenseCategories.value, year.value),
);
const balanceMonths = computed(() =>
  paymentMethodBalanceTrend(incomes.value, expenses.value, paymentMethods.value, year.value),
);

/** New transactions default to this month in the current year, and to January otherwise. */
const recordMonth = computed(() =>
  year.value === currentYear() ? currentMonth() : monthOfYear(year.value, 1),
);
const {
  formOpen,
  editingExpense,
  editingIncome,
  exchangePreview,
  saving,
  initialDate,
  openNew,
  closeForm,
  save,
} = useTransactionEditor({ month: recordMonth, expenses, incomes });
</script>

<template>
  <div class="flex min-w-0 flex-1">
    <UDashboardPanel id="annual">
      <template #header>
        <UDashboardNavbar title="年間集計">
          <template #leading>
            <UDashboardSidebarCollapse />
          </template>
          <template #right>
            <YearNavigator
              :model-value="year"
              :years="availableYears"
              @update:model-value="setYear"
            />
            <RecordTransactionButton @click="openNew" />
            <UColorModeButton />
          </template>
        </UDashboardNavbar>
      </template>
      <template #body>
        <DataLoadState :loading="loading" :error="loadError" @retry="load">
          <div class="space-y-6">
            <AnnualSummaryPanel :income="incomeTotal" :expense="expenseTotal" />

            <div class="grid gap-6 lg:grid-cols-[1.6fr_1fr]">
              <AnnualCashflowChart :months="projectedMonths" />
              <AnnualSpendingBreakdown :spending="spending" :total="expenseTotal" />
            </div>

            <div class="grid gap-6 xl:grid-cols-2">
              <AnnualCategoryTrendChart :months="categoryMonths" />
              <AnnualBalanceTrendChart :months="balanceMonths" />
            </div>

            <AnnualMonthlyTable :months="projectedMonths" />
            <AnnualCategoryTable :rows="categoryRows" />
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
      :exchange-preview="exchangePreview"
      :on-submit="save"
      @update:open="
        (value) => {
          if (!value) closeForm();
        }
      "
    />
  </div>
</template>
