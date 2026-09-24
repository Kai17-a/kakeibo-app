<script setup lang="ts">
import type { Category, PaymentMethod, RecurringExpense } from '~/types/settings'
import type { ExchangeRatePreview, Expense, ExpenseInput, Income, IncomeInput } from '~/types/transactions'
import { categoryTotals, dailyCategoryTotals, inPeriod, sumAmounts } from '~/utils/summaries'
import { filterExpenses, filterIncomes } from '~/utils/filters'
import { currentMonth, formatCurrency, formatDate, isValidMonth, shiftMonth } from '~/utils/format'

useSeoMeta({ title: '日別集計' })

const route = useRoute()
const month = computed(() => (typeof route.query.month === 'string' && isValidMonth(route.query.month) ? route.query.month : currentMonth()))
function setMonth(next: string) {
  navigateTo({ path: '/daily', query: { month: next } }, { replace: true })
}
const monthLabel = computed(() => new Intl.DateTimeFormat('ja-JP', { year: 'numeric', month: 'long' }).format(new Date(`${month.value}-01T00:00:00`)))

const settingsApi = useSettingsApi()
const transactionsApi = useTransactionsApi()

const expenses = ref<Expense[]>([])
const incomes = ref<Income[]>([])
const expenseCategories = ref<Category[]>([])
const incomeCategories = ref<Category[]>([])
const paymentMethods = ref<PaymentMethod[]>([])
const recurringExpenses = ref<RecurringExpense[]>([])
const loading = ref(true)
const loadError = ref('')

async function loadAll() {
  loading.value = true
  loadError.value = ''
  try {
    const [expenseData, incomeData, expenseCategoryData, incomeCategoryData, paymentData, recurringData] = await Promise.all([
      transactionsApi.expenses(),
      transactionsApi.incomes(),
      settingsApi.categories('expense').list(),
      settingsApi.categories('income').list(),
      settingsApi.paymentMethods.list(),
      settingsApi.recurringExpenses.list()
    ])
    expenses.value = expenseData
    incomes.value = incomeData
    expenseCategories.value = expenseCategoryData
    incomeCategories.value = incomeCategoryData
    paymentMethods.value = paymentData
    recurringExpenses.value = recurringData
  } catch (error) {
    loadError.value = apiErrorMessage(error)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const monthExpenses = computed(() => inPeriod(expenses.value, month.value))
const monthIncomes = computed(() => inPeriod(incomes.value, month.value))

const tab = ref<'summary' | 'details' | 'income-details' | 'categories'>('summary')
const tabItems = [
  { label: '収支・明細', value: 'summary' },
  { label: '支出明細', value: 'details' },
  { label: '収入明細', value: 'income-details' },
  { label: '月ごとのカテゴリ別支出', value: 'categories' }
]

const keyword = ref('')
const categoryId = ref('')
const paymentMethodId = ref('')
const incomeKeyword = ref('')
const incomeCategoryId = ref('')
const incomePaymentMethodId = ref('')
const ALL_FILTER_VALUE = '__ALL__'

function clearFilters() {
  keyword.value = ''
  categoryId.value = ''
  paymentMethodId.value = ''
}
function clearIncomeFilters() {
  incomeKeyword.value = ''
  incomeCategoryId.value = ''
  incomePaymentMethodId.value = ''
}

const incomeTotal = computed(() => sumAmounts(monthIncomes.value))
const expenseTotal = computed(() => sumAmounts(monthExpenses.value))
const paymentNames = computed(() => new Map(paymentMethods.value.map(item => [item.id, item.name])))
const categoryNames = computed(() => new Map(expenseCategories.value.map(item => [item.id, item.name])))
const incomeCategoryNames = computed(() => new Map(incomeCategories.value.map(item => [item.id, item.name])))

const incomeBreakdown = computed(() => incomeCategories.value.map(category => ({
  ...category,
  total: sumAmounts(monthIncomes.value.filter(item => item.category_id === category.id))
})))
const paymentBreakdown = computed(() => paymentMethods.value.map(method => ({
  ...method,
  total: sumAmounts(monthExpenses.value.filter(item => item.payment_method_id === method.id))
})))

const recurring = computed(() => recurringExpenses.value.filter(item =>
  item.is_active && !item.is_variable && item.start_date.slice(0, 7) <= month.value && (!item.end_date || item.end_date.slice(0, 7) >= month.value)
))
const fixedRecurringExpenseIds = computed(() => new Set(recurring.value.map(item => item.id)))
const variableRecurringCategoryIds = computed(() => new Set(
  recurringExpenses.value
    .filter(item => item.is_active && item.is_variable === true && item.start_date.slice(0, 7) <= month.value && (!item.end_date || item.end_date.slice(0, 7) >= month.value))
    .map(item => item.category_id)
))
function isLinkedToFixedRecurring(item: Expense) {
  return Boolean(item.recurring_expense_id && fixedRecurringExpenseIds.value.has(item.recurring_expense_id))
}
const fixedVariable = computed(() => monthExpenses.value.filter(item => !isLinkedToFixedRecurring(item) && variableRecurringCategoryIds.value.has(item.category_id)))
const variable = computed(() => monthExpenses.value.filter(item => !isLinkedToFixedRecurring(item) && !variableRecurringCategoryIds.value.has(item.category_id)))
const recurringTotal = computed(() => sumAmounts(recurring.value))
const fixedVariableTotal = computed(() => sumAmounts(fixedVariable.value))
const fixedVariableTotals = computed(() => categoryTotals(fixedVariable.value, expenseCategories.value).filter(item => item.total))
const variableTotal = computed(() => sumAmounts(variable.value))
const variableTotals = computed(() => categoryTotals(variable.value, expenseCategories.value).filter(item => item.total))

const ledger = computed(() => filterExpenses(monthExpenses.value, { keyword: keyword.value, categoryId: categoryId.value, paymentMethodId: paymentMethodId.value }).sort((a, b) => a.transaction_date.localeCompare(b.transaction_date)))
const incomeLedger = computed(() => filterIncomes(monthIncomes.value, { keyword: incomeKeyword.value, categoryId: incomeCategoryId.value, paymentMethodId: incomePaymentMethodId.value }).sort((a, b) => a.transaction_date.localeCompare(b.transaction_date)))

const dailyRows = computed(() => dailyCategoryTotals(monthExpenses.value, month.value))
const activeExpenseCategories = computed(() => expenseCategories.value.filter(category => monthExpenses.value.some(item => item.category_id === category.id)))

// --- transaction registration / edit modal ---
const transactionModalOpen = ref(false)
const editingExpense = ref<Expense | null>(null)
const editingIncome = ref<Income | null>(null)
const exchangePreview = ref<ExchangeRatePreview | null>(null)
const saving = ref(false)
const toast = useToast()
const initialTransactionDate = computed(() => `${month.value}-${String(Math.min(new Date().getDate(), 28)).padStart(2, '0')}`)

function editExpense(item: Expense) {
  editingExpense.value = item
  editingIncome.value = null
  exchangePreview.value = null
  transactionModalOpen.value = true
}
function editIncome(item: Income) {
  editingExpense.value = null
  editingIncome.value = item
  exchangePreview.value = null
  transactionModalOpen.value = true
}
function closeTransaction() {
  transactionModalOpen.value = false
  editingExpense.value = null
  editingIncome.value = null
  exchangePreview.value = null
}
async function saveTransaction(kind: 'expense' | 'income', input: ExpenseInput | IncomeInput, keepOpen: boolean): Promise<boolean> {
  saving.value = true
  try {
    if (kind === 'expense' && editingExpense.value) {
      const updated = await transactionsApi.updateExpense(editingExpense.value.id, input as ExpenseInput)
      expenses.value = expenses.value.map(item => item.id === updated.id ? updated : item)
      toast.add({ title: '支出を更新しました', color: 'success' })
    } else if (kind === 'income' && editingIncome.value) {
      const updated = await transactionsApi.updateIncome(editingIncome.value.id, input as IncomeInput)
      incomes.value = incomes.value.map(item => item.id === updated.id ? updated : item)
      toast.add({ title: '収入を更新しました', color: 'success' })
    } else if (kind === 'expense') {
      expenses.value = [await transactionsApi.createExpense(input as ExpenseInput), ...expenses.value]
      toast.add({ title: '支出を登録しました', color: 'success' })
    } else {
      incomes.value = [await transactionsApi.createIncome(input as IncomeInput), ...incomes.value]
      toast.add({ title: '収入を登録しました', color: 'success' })
    }
    if (!keepOpen) closeTransaction()
    return true
  } catch (error) {
    toast.add({ title: apiErrorMessage(error), color: 'error' })
    return false
  } finally {
    saving.value = false
  }
}

// --- delete confirmation ---
type DeleteTarget = { type: 'expense', item: Expense } | { type: 'income', item: Income }
const deleteTarget = ref<DeleteTarget | null>(null)
const deleteOpen = computed({
  get: () => deleteTarget.value !== null,
  set: (value: boolean) => { if (!value) deleteTarget.value = null }
})
function askDeleteExpense(item: Expense) {
  deleteTarget.value = { type: 'expense', item }
}
function askDeleteIncome(item: Income) {
  deleteTarget.value = { type: 'income', item }
}
const removing = ref(false)
async function confirmDelete() {
  const target = deleteTarget.value
  if (!target) return
  removing.value = true
  try {
    if (target.type === 'expense') {
      await transactionsApi.deleteExpense(target.item.id)
      expenses.value = expenses.value.filter(item => item.id !== target.item.id)
    } else {
      await transactionsApi.deleteIncome(target.item.id)
      incomes.value = incomes.value.filter(item => item.id !== target.item.id)
    }
    toast.add({ title: '明細を削除しました', color: 'success' })
    deleteTarget.value = null
  } catch (error) {
    toast.add({ title: apiErrorMessage(error), color: 'error' })
  } finally {
    removing.value = false
  }
}
</script>

<template>
  <div class="flex min-w-0 flex-1">
    <UDashboardPanel id="daily">
      <template #header>
        <UDashboardNavbar title="日別集計">
          <template #leading>
            <UDashboardSidebarCollapse />
          </template>
        </UDashboardNavbar>
        <UDashboardToolbar>
          <template #left>
            <div class="flex min-w-0 items-baseline gap-2">
              <h2 class="truncate font-semibold">
                {{ monthLabel }} 家計簿
              </h2>
              <span class="hidden shrink-0 text-sm text-muted sm:inline">単位：円</span>
            </div>
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
                class="w-44 sm:w-40"
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
          </template>
        </UDashboardToolbar>
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
        <div
          v-if="loading"
          class="space-y-4"
        >
          <USkeleton class="h-28 w-full" />
          <USkeleton class="h-28 w-full" />
        </div>
        <UCard v-else>
          <UTabs
            v-model="tab"
            :items="tabItems"
            :ui="{
              list: 'overflow-x-auto',
              trigger: 'shrink-0',
              label: 'overflow-visible text-clip whitespace-nowrap'
            }"
            class="mb-4"
          />

          <div v-if="tab === 'categories'">
            <h3 class="mb-2 bg-elevated px-3 py-2 text-sm font-bold">
              日ごとのカテゴリ別支出
            </h3>
            <div
              v-if="activeExpenseCategories.length"
              class="overflow-x-auto"
            >
              <table class="w-full min-w-[760px] border-collapse text-sm">
                <thead class="bg-elevated">
                  <tr>
                    <th class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                      日付
                    </th>
                    <th
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="min-w-24 px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase"
                    >
                      {{ category.name }}
                    </th>
                    <th class="sticky right-0 bg-elevated px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase">
                      合計
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="day in dailyRows"
                    :key="day.date"
                    class="border-t border-default"
                  >
                    <th class="sticky left-0 w-32 min-w-32 bg-default px-3 py-2 text-left whitespace-nowrap">
                      {{ formatDate(day.date) }}
                    </th>
                    <td
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="px-3 py-2 text-right"
                    >
                      {{ day.values.get(category.id)?.toLocaleString('ja-JP') ?? '' }}
                    </td>
                    <td class="sticky right-0 bg-elevated px-3 py-2 text-right font-bold">
                      {{ day.total ? day.total.toLocaleString('ja-JP') : '' }}
                    </td>
                  </tr>
                </tbody>
                <tfoot class="border-t-2 border-default bg-elevated">
                  <tr>
                    <th class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-3 text-left whitespace-nowrap">
                      合計
                    </th>
                    <td
                      v-for="category in activeExpenseCategories"
                      :key="category.id"
                      class="px-3 py-3 text-right font-bold"
                    >
                      {{ sumAmounts(monthExpenses.filter(item => item.category_id === category.id)).toLocaleString('ja-JP') }}
                    </td>
                    <td class="sticky right-0 bg-elevated px-3 py-3 text-right font-bold">
                      {{ sumAmounts(monthExpenses).toLocaleString('ja-JP') }}
                    </td>
                  </tr>
                </tfoot>
              </table>
            </div>
            <p
              v-else
              class="py-10 text-center text-sm text-muted"
            >
              この月の支出はまだありません。
            </p>
          </div>

          <div
            v-else-if="tab === 'details'"
            class="min-w-0"
          >
            <h3 class="bg-elevated px-3 py-2 text-sm font-bold">
              支出明細
            </h3>
            <div class="grid gap-4 border-b border-default py-4 sm:grid-cols-2 xl:grid-cols-[minmax(12rem,1fr)_12rem_12rem_auto] xl:items-end">
              <UFormField label="備考を検索">
                <UInput
                  v-model="keyword"
                  type="search"
                  placeholder="キーワードを入力"
                  class="w-full"
                />
              </UFormField>
              <UFormField label="カテゴリ">
                <USelect
                  :model-value="categoryId || ALL_FILTER_VALUE"
                  :items="[{ label: 'すべて', value: ALL_FILTER_VALUE }, ...expenseCategories.map(category => ({ label: category.name, value: category.id }))]"
                  class="w-full"
                  @update:model-value="categoryId = $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)"
                />
              </UFormField>
              <UFormField label="支払方法">
                <USelect
                  :model-value="paymentMethodId || ALL_FILTER_VALUE"
                  :items="[{ label: 'すべて', value: ALL_FILTER_VALUE }, ...paymentMethods.map(method => ({ label: method.name, value: method.id }))]"
                  class="w-full"
                  @update:model-value="paymentMethodId = $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)"
                />
              </UFormField>
              <UButton
                color="neutral"
                variant="outline"
                @click="clearFilters"
              >
                条件をクリア
              </UButton>
            </div>
            <div class="max-h-[38rem] overflow-auto">
              <table class="w-full min-w-[800px] text-sm">
                <thead class="sticky top-0 bg-default">
                  <tr class="border-b border-default">
                    <th class="w-32 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                      日付
                    </th>
                    <th class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
                      摘要
                    </th>
                    <th class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
                      支払方法
                    </th>
                    <th class="w-32 px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                      金額
                    </th>
                    <th class="min-w-40 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
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
                    <td class="px-3 py-2 text-right whitespace-nowrap">
                      {{ Number(item.amount).toLocaleString('ja-JP') }}
                    </td>
                    <td class="px-3 py-2">
                      {{ item.description ?? '' }}
                    </td>
                    <td class="px-3 py-2">
                      <div class="flex justify-end gap-2">
                        <UButton
                          color="neutral"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${categoryNames.get(item.category_id) ?? ''}を編集`"
                          @click="editExpense(item)"
                        >
                          編集
                        </UButton>
                        <UButton
                          color="error"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${categoryNames.get(item.category_id) ?? ''}を削除`"
                          @click="askDeleteExpense(item)"
                        >
                          削除
                        </UButton>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
              <p
                v-if="!ledger.length"
                class="py-12 text-center text-sm text-muted"
              >
                {{ monthExpenses.length ? '条件に一致する明細がありません。' : 'この月の支出明細はありません。' }}
              </p>
            </div>
          </div>

          <div
            v-else-if="tab === 'income-details'"
            class="min-w-0"
          >
            <h3 class="bg-elevated px-3 py-2 text-sm font-bold">
              収入明細
            </h3>
            <div class="grid gap-4 border-b border-default py-4 sm:grid-cols-2 xl:grid-cols-[minmax(12rem,1fr)_12rem_12rem_auto] xl:items-end">
              <UFormField label="備考を検索">
                <UInput
                  v-model="incomeKeyword"
                  type="search"
                  placeholder="キーワードを入力"
                  class="w-full"
                />
              </UFormField>
              <UFormField label="カテゴリ">
                <USelect
                  :model-value="incomeCategoryId || ALL_FILTER_VALUE"
                  :items="[{ label: 'すべて', value: ALL_FILTER_VALUE }, ...incomeCategories.map(category => ({ label: category.name, value: category.id }))]"
                  class="w-full"
                  @update:model-value="incomeCategoryId = $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)"
                />
              </UFormField>
              <UFormField label="支払方法">
                <USelect
                  :model-value="incomePaymentMethodId || ALL_FILTER_VALUE"
                  :items="[{ label: 'すべて', value: ALL_FILTER_VALUE }, ...paymentMethods.map(method => ({ label: method.name, value: method.id }))]"
                  class="w-full"
                  @update:model-value="incomePaymentMethodId = $event == null || $event === ALL_FILTER_VALUE ? '' : String($event)"
                />
              </UFormField>
              <UButton
                color="neutral"
                variant="outline"
                @click="clearIncomeFilters"
              >
                条件をクリア
              </UButton>
            </div>
            <div class="max-h-[38rem] overflow-auto">
              <table class="w-full min-w-[800px] text-sm">
                <thead class="sticky top-0 bg-default">
                  <tr class="border-b border-default">
                    <th class="w-32 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                      日付
                    </th>
                    <th class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
                      摘要
                    </th>
                    <th class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
                      支払方法
                    </th>
                    <th class="w-32 px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                      金額
                    </th>
                    <th class="min-w-40 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase">
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
                      {{ item.payment_method_id ? (paymentNames.get(item.payment_method_id) ?? '—') : '—' }}
                    </td>
                    <td class="px-3 py-2 text-right whitespace-nowrap">
                      {{ Number(item.amount).toLocaleString('ja-JP') }}
                    </td>
                    <td class="px-3 py-2">
                      {{ item.description ?? '' }}
                    </td>
                    <td class="px-3 py-2">
                      <div class="flex justify-end gap-2">
                        <UButton
                          color="neutral"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${incomeCategoryNames.get(item.category_id) ?? ''}を編集`"
                          @click="editIncome(item)"
                        >
                          編集
                        </UButton>
                        <UButton
                          color="error"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${incomeCategoryNames.get(item.category_id) ?? ''}を削除`"
                          @click="askDeleteIncome(item)"
                        >
                          削除
                        </UButton>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
              <p
                v-if="!incomeLedger.length"
                class="py-12 text-center text-sm text-muted"
              >
                {{ monthIncomes.length ? '条件に一致する明細がありません。' : 'この月の収入明細はありません。' }}
              </p>
            </div>
          </div>

          <div
            v-else
            class="grid gap-6 xl:grid-cols-[18rem_1fr]"
          >
            <aside class="border-default xl:border-r xl:pr-4">
              <h3 class="bg-elevated px-3 py-2 text-sm font-bold">
                収支サマリー
              </h3>
              <dl class="grid grid-cols-2 text-sm">
                <dt class="p-2">
                  収入
                </dt>
                <dd class="p-2 text-right">
                  {{ formatCurrency(incomeTotal) }}
                </dd>
                <dt class="p-2">
                  支出
                </dt>
                <dd class="p-2 text-right">
                  {{ formatCurrency(expenseTotal) }}
                </dd>
                <dt class="p-2 font-bold">
                  収支
                </dt>
                <dd class="p-2 text-right font-bold">
                  {{ formatCurrency(incomeTotal - expenseTotal) }}
                </dd>
              </dl>
              <h3 class="mt-4 bg-elevated px-3 py-2 text-sm font-bold">
                収入
              </h3>
              <div
                v-for="item in incomeBreakdown"
                :key="item.id"
                class="flex justify-between p-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span>{{ item.total.toLocaleString('ja-JP') }}</span>
              </div>
              <h3 class="mt-4 bg-elevated px-3 py-2 text-sm font-bold">
                支払種別
              </h3>
              <div
                v-for="item in paymentBreakdown"
                :key="item.id"
                class="flex justify-between p-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span>{{ item.total.toLocaleString('ja-JP') }}</span>
              </div>
            </aside>
            <div>
              <div class="flex items-center justify-between gap-4 bg-elevated px-3 py-2">
                <h3 class="text-sm font-bold">
                  支出（固定費）
                </h3>
                <span class="text-sm font-bold whitespace-nowrap">{{ formatCurrency(recurringTotal) }}</span>
              </div>
              <div
                v-for="item in recurring"
                :key="item.id"
                class="flex items-center justify-between border-b border-default p-2 text-sm"
              >
                <span>
                  {{ item.name }}
                  <small class="block text-muted">毎月{{ item.payment_day }}日</small>
                </span>
                <span>{{ Number(item.amount).toLocaleString('ja-JP') }}</span>
              </div>
              <div class="mt-4 flex items-center justify-between gap-4 bg-elevated px-3 py-2">
                <h3 class="text-sm font-bold">
                  支出（固定変動費）
                </h3>
                <span class="text-sm font-bold whitespace-nowrap">{{ formatCurrency(fixedVariableTotal) }}</span>
              </div>
              <div
                v-for="item in fixedVariableTotals"
                :key="item.id"
                class="flex justify-between border-b border-default p-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span>{{ item.total.toLocaleString('ja-JP') }}</span>
              </div>
              <div class="mt-4 flex items-center justify-between gap-4 bg-elevated px-3 py-2">
                <h3 class="text-sm font-bold">
                  支出（変動費）
                </h3>
                <span class="text-sm font-bold whitespace-nowrap">{{ formatCurrency(variableTotal) }}</span>
              </div>
              <div
                v-for="item in variableTotals"
                :key="item.id"
                class="flex justify-between border-b border-default p-2 text-sm"
              >
                <span>{{ item.name }}</span>
                <span>{{ item.total.toLocaleString('ja-JP') }}</span>
              </div>
            </div>
          </div>
        </UCard>
      </template>
    </UDashboardPanel>

    <TransactionsTransactionFormModal
      v-if="transactionModalOpen"
      v-model:open="transactionModalOpen"
      :expense-categories="expenseCategories"
      :income-categories="incomeCategories"
      :payment-methods="paymentMethods"
      :saving="saving"
      :initial-date="initialTransactionDate"
      :initial-expense="editingExpense"
      :initial-income="editingIncome"
      :exchange-preview="exchangePreview"
      :on-submit="saveTransaction"
      @update:open="(value) => { if (!value) closeTransaction() }"
    />

    <UModal
      v-model:open="deleteOpen"
      title="削除の確認"
      :description="deleteTarget ? `この${deleteTarget.type === 'expense' ? '支出' : '収入'}明細を削除しますか？` : ''"
      :dismissible="!removing"
      :close="!removing"
      :ui="{ footer: 'justify-end' }"
    >
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          :disabled="removing"
          @click="deleteOpen = false"
        >
          キャンセル
        </UButton>
        <UButton
          color="error"
          :loading="removing"
          :disabled="removing"
          @click="confirmDelete"
        >
          削除
        </UButton>
      </template>
    </UModal>
  </div>
</template>
