<script setup lang="ts">
import { inPeriod } from "~/utils/summaries";
import { filterExpenses, filterIncomes } from "~/utils/filters";

useSeoMeta({ title: "日別集計" });

const { month, setMonth } = useMonthQuery("/daily");
const {
  expenses,
  incomes,
  expenseCategories,
  incomeCategories,
  paymentMethods,
  recurringExpenses,
  loading,
  loadError,
  load,
} = useFinanceData({ incomeCategories: true });

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

const categoryNames = computed(
  () => new Map(expenseCategories.value.map((item) => [item.id, item.name])),
);
const incomeCategoryNames = computed(
  () => new Map(incomeCategories.value.map((item) => [item.id, item.name])),
);
const paymentNames = computed(
  () => new Map(paymentMethods.value.map((item) => [item.id, item.name])),
);

const byDate = (a: { transaction_date: string }, b: { transaction_date: string }) =>
  a.transaction_date.localeCompare(b.transaction_date);
const ledger = computed(() =>
  filterExpenses(monthExpenses.value, {
    keyword: keyword.value,
    categoryId: categoryId.value,
    paymentMethodId: paymentMethodId.value,
  }).toSorted(byDate),
);
const incomeLedger = computed(() =>
  filterIncomes(monthIncomes.value, {
    keyword: incomeKeyword.value,
    categoryId: incomeCategoryId.value,
    paymentMethodId: incomePaymentMethodId.value,
  }).toSorted(byDate),
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
            <MonthNavigator :model-value="month" @update:model-value="setMonth" />
            <RecordTransactionButton @click="openNew" />
          </template>
        </UDashboardNavbar>
      </template>
      <template #body>
        <DataLoadState :loading="loading" :error="loadError" @retry="load">
          <UCard class="shrink-0">
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

            <DailyCategoryTable
              v-if="tab === 'categories'"
              :month="month"
              :expenses="monthExpenses"
              :categories="expenseCategories"
            />

            <div v-else-if="tab === 'details'" class="min-w-0">
              <h2 class="text-base font-semibold">支出明細</h2>
              <DailyLedgerFilters
                v-model:keyword="keyword"
                v-model:category-id="categoryId"
                v-model:payment-method-id="paymentMethodId"
                :categories="expenseCategories"
                :payment-methods="paymentMethods"
              />
              <DailyLedgerTable
                kind="expense"
                :items="ledger"
                :empty-text="
                  monthExpenses.length
                    ? '条件に一致する明細がありません。'
                    : 'この月の支出明細はありません。'
                "
                :category-names="categoryNames"
                :payment-names="paymentNames"
                @edit="editExpense"
                @delete="askDeleteExpense"
              />
            </div>

            <div v-else-if="tab === 'income-details'" class="min-w-0">
              <h2 class="text-base font-semibold">収入明細</h2>
              <DailyLedgerFilters
                v-model:keyword="incomeKeyword"
                v-model:category-id="incomeCategoryId"
                v-model:payment-method-id="incomePaymentMethodId"
                :categories="incomeCategories"
                :payment-methods="paymentMethods"
              />
              <DailyLedgerTable
                kind="income"
                :items="incomeLedger"
                :empty-text="
                  monthIncomes.length
                    ? '条件に一致する明細がありません。'
                    : 'この月の収入明細はありません。'
                "
                :category-names="incomeCategoryNames"
                :payment-names="paymentNames"
                @edit="editIncome"
                @delete="askDeleteIncome"
              />
            </div>

            <DailyMonthlyBreakdown
              v-else
              :month="month"
              :expenses="monthExpenses"
              :incomes="monthIncomes"
              :recurring-expenses="recurringExpenses"
              :expense-categories="expenseCategories"
              :income-categories="incomeCategories"
              :payment-methods="paymentMethods"
            />
          </UCard>
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

    <ConfirmDeleteModal
      v-model:open="deleteOpen"
      :description="deleteDescription"
      :busy="removing"
      @confirm="confirmDelete"
    />
  </div>
</template>
