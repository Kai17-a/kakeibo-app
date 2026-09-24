<script setup lang="ts">
import { annualCategoryRows, monthOfYear, projectedAnnualMonths } from "~/utils/annual";
import { categoryMonthlyTotals, paymentMethodBalanceTrend } from "~/utils/summaries";

useSeoMeta({ title: "年間集計" });

const { year, setYear } = useYearQuery("/annual");

const {
  expenses,
  incomes,
  expenseCategories,
  paymentMethods,
  recurringExpenses,
  recurringIncomes,
  loading,
  loadError,
  load,
} = useFinanceData({ recurringIncomes: true });

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
</script>

<template>
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
</template>
