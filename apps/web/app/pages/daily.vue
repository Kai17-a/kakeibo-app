<script setup lang="ts">
import { inPeriod } from "~/utils/summaries";
import { filterExpenses, filterIncomes } from "~/utils/filters";

useSeoMeta({ title: "日別集計" });

const { month, setMonth } = useMonthQuery("/daily");
const {
  expenses,
  incomes,
  transfers,
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
const monthTransfers = computed(() => inPeriod(transfers.value, month.value).toSorted(byDate));

const tab = ref<"summary" | "details" | "income-details" | "transfer-details" | "categories">(
  "summary",
);
const tabItems = [
  { label: "収支・明細", value: "summary" },
  { label: "支出明細", value: "details" },
  { label: "収入明細", value: "income-details" },
  { label: "振替明細", value: "transfer-details" },
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
    <UDashboardPanel id="daily">
      <template #header>
        <UDashboardNavbar title="日別集計">
          <template #leading>
            <UDashboardSidebarCollapse />
          </template>
          <template #right>
            <MonthNavigator :model-value="month" @update:model-value="setMonth" />
            <TransferButton @click="openNewTransfer" />
            <RecordTransactionButton @click="openNew" />
            <UColorModeButton />
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

            <div v-else-if="tab === 'transfer-details'" class="min-w-0">
              <h2 class="mb-4 text-base font-semibold">振替明細</h2>
              <div class="overflow-auto">
                <table class="w-full min-w-[640px] text-sm">
                  <thead>
                    <tr class="border-b border-default">
                      <th class="px-3 py-2 text-left">日付</th>
                      <th class="px-3 py-2 text-left">移動</th>
                      <th class="px-3 py-2 text-right">金額</th>
                      <th class="px-3 py-2 text-left">メモ</th>
                      <th><span class="sr-only">操作</span></th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr
                      v-for="item in monthTransfers"
                      :key="item.id"
                      class="border-t border-default"
                    >
                      <td class="px-3 py-2">{{ item.transaction_date }}</td>
                      <td class="px-3 py-2">
                        {{ paymentNames.get(item.from_payment_method_id) }} →
                        {{ paymentNames.get(item.to_payment_method_id) }}
                      </td>
                      <td class="px-3 py-2 text-right tabular-nums">
                        {{ Number(item.amount).toLocaleString("ja-JP") }}円
                      </td>
                      <td class="px-3 py-2">{{ item.description ?? "" }}</td>
                      <td class="px-3 py-2 text-right">
                        <RowActionsMenu
                          :label="`${item.transaction_date} 振替`"
                          @edit="editTransfer(item)"
                          @delete="askDeleteTransfer(item)"
                        />
                      </td>
                    </tr>
                  </tbody>
                </table>
                <div v-if="!monthTransfers.length" class="py-12 text-center text-sm text-muted">
                  <p>この月の振替明細はありません。</p>
                  <UButton
                    icon="i-lucide-arrow-right-left"
                    color="neutral"
                    variant="outline"
                    class="mx-auto mt-4 flex w-fit"
                    @click="openNewTransfer"
                  >
                    振替を登録
                  </UButton>
                </div>
              </div>
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
