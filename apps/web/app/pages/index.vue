<script setup lang="ts">
import type { Budget, Category, PaymentMethod, RecurringExpense, RecurringIncome } from '~/types/settings'
import type { ExchangeRatePreview, Expense, ExpenseInput, Income, IncomeInput } from '~/types/transactions'
import { budgetActuals, categoryTotals, inPeriod, mergeTransactions, recurringForecast, sumAmounts } from '~/utils/summaries'
import { currentMonth, formatCurrency, formatDate, formatSignedCurrency, isValidMonth, shiftMonth } from '~/utils/format'

useSeoMeta({ title: '月間集計' })

const route = useRoute()
const month = computed(() => (typeof route.query.month === 'string' && isValidMonth(route.query.month) ? route.query.month : currentMonth()))
function setMonth(next: string) {
  navigateTo({ path: '/', query: { month: next } }, { replace: true })
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
const recurringIncomes = ref<RecurringIncome[]>([])
const budgets = ref<Budget[]>([])
const loading = ref(true)
const loadError = ref('')

async function loadAll() {
  loading.value = true
  loadError.value = ''
  try {
    const [expenseData, incomeData, expenseCategoryData, incomeCategoryData, paymentData, recurringData, recurringIncomeData, budgetData] = await Promise.all([
      transactionsApi.expenses(),
      transactionsApi.incomes(),
      settingsApi.categories('expense').list(),
      settingsApi.categories('income').list(),
      settingsApi.paymentMethods.list(),
      settingsApi.recurringExpenses.list(),
      settingsApi.recurringIncomes.list(),
      settingsApi.budgets.list()
    ])
    expenses.value = expenseData
    incomes.value = incomeData
    expenseCategories.value = expenseCategoryData
    incomeCategories.value = incomeCategoryData
    paymentMethods.value = paymentData
    recurringExpenses.value = recurringData
    recurringIncomes.value = recurringIncomeData
    budgets.value = budgetData
  } catch (error) {
    loadError.value = apiErrorMessage(error)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const monthExpenses = computed(() => inPeriod(expenses.value, month.value))
const monthIncomes = computed(() => inPeriod(incomes.value, month.value))

const usdRecurringPreviews = ref(new Map<string, ExchangeRatePreview>())
let usdPreviewRequestId = 0
watch([month, recurringExpenses], async ([currentMonthValue]) => {
  const requestId = ++usdPreviewRequestId
  const targets = recurringExpenses.value.filter(item => item.is_active && !item.is_variable && item.currency_code === 'USD')
  if (!targets.length) {
    if (requestId === usdPreviewRequestId) usdRecurringPreviews.value = new Map()
    return
  }
  const results = await Promise.allSettled(
    targets.map(async item => [item.id, await transactionsApi.previewRecurringExpenseExchangeRate(item.id, currentMonthValue)] as const)
  )
  if (requestId !== usdPreviewRequestId) return
  usdRecurringPreviews.value = new Map(results.flatMap(result => result.status === 'fulfilled' ? [result.value] : []))
}, { immediate: true })

const expenseTotal = computed(() => sumAmounts(monthExpenses.value))
const incomeTotal = computed(() => sumAmounts(monthIncomes.value))
const forecast = computed(() => recurringForecast(monthExpenses.value, monthIncomes.value, recurringExpenses.value, recurringIncomes.value, month.value, usdRecurringPreviews.value))
const projectedExpenseTotal = computed(() => expenseTotal.value + forecast.value.expense)
const projectedIncomeTotal = computed(() => incomeTotal.value + forecast.value.income)
const balance = computed(() => projectedIncomeTotal.value - projectedExpenseTotal.value)
const transactions = computed(() => mergeTransactions(monthExpenses.value, monthIncomes.value))
const transactionGroups = computed(() => {
  const groups = new Map<string, typeof transactions.value>()
  for (const item of transactions.value) {
    const group = groups.get(item.transaction_date) ?? []
    group.push(item)
    groups.set(item.transaction_date, group)
  }
  return [...groups].map(([date, items]) => ({
    date,
    items,
    total: items.reduce((sum, item) => sum + (item.kind === 'income' ? Number(item.amount) : -Number(item.amount)), 0)
  }))
})

const fixedRecurring = computed(() => recurringExpenses.value.filter(item => item.is_active && !item.is_variable))
const variableRecurring = computed(() => recurringExpenses.value.filter(item => item.is_active && item.is_variable))
const fixedRecurringIncomes = computed(() => recurringIncomes.value.filter(item => item.is_active && !item.is_variable))
const variableRecurringIncomes = computed(() => recurringIncomes.value.filter(item => item.is_active && item.is_variable))

const expenseNames = computed(() => new Map(expenseCategories.value.map(item => [item.id, item.name])))
const incomeNames = computed(() => new Map(incomeCategories.value.map(item => [item.id, item.name])))
const paymentNames = computed(() => new Map(paymentMethods.value.map(item => [item.id, item.name])))

const spending = computed(() => categoryTotals(monthExpenses.value, expenseCategories.value).filter(item => item.total).sort((a, b) => b.total - a.total))
const actuals = computed(() => budgetActuals(monthExpenses.value, expenseCategories.value, budgets.value, month.value))
const budgetTotal = computed(() => actuals.value.reduce((sum, item) => sum + item.budget, 0))
const budgetSpent = computed(() => actuals.value.reduce((sum, item) => sum + item.actual, 0))
const budgetRate = computed(() => budgetTotal.value ? (budgetSpent.value / budgetTotal.value) * 100 : 0)
function transactionLabel(item: ReturnType<typeof mergeTransactions>[number]) {
  return item.description || (item.kind === 'expense' ? expenseNames.value.get(item.category_id) : incomeNames.value.get(item.category_id)) || '名称なし'
}

function transactionMeta(item: ReturnType<typeof mergeTransactions>[number]) {
  const category = item.kind === 'expense' ? expenseNames.value.get(item.category_id) : incomeNames.value.get(item.category_id)
  const payment = item.payment_method_id ? paymentNames.value.get(item.payment_method_id) : undefined
  const label = transactionLabel(item)
  return [label === category ? undefined : category, payment].filter(Boolean).join(' · ')
}

function transactionActions(item: ReturnType<typeof mergeTransactions>[number]) {
  return [[{
    label: '編集',
    icon: 'i-lucide-pencil',
    onSelect: () => item.kind === 'expense' ? editExpense(item) : editIncome(item)
  }, {
    label: '削除',
    icon: 'i-lucide-trash-2',
    color: 'error' as const,
    onSelect: () => item.kind === 'expense' ? askDeleteExpense(item) : askDeleteIncome(item)
  }]]
}

// --- transaction registration modal ---
const transactionModalOpen = ref(false)
const editingExpense = ref<Expense | null>(null)
const editingIncome = ref<Income | null>(null)
const transactionPreset = ref<RecurringExpense | null>(null)
const incomeTransactionPreset = ref<RecurringIncome | null>(null)
const exchangePreview = ref<ExchangeRatePreview | null>(null)
const saving = ref(false)
const toast = useToast()

const initialTransactionDate = computed(() => `${month.value}-${String(Math.min(new Date().getDate(), 28)).padStart(2, '0')}`)

function openTransaction() {
  editingExpense.value = null
  editingIncome.value = null
  transactionPreset.value = null
  incomeTransactionPreset.value = null
  exchangePreview.value = null
  transactionModalOpen.value = true
}
function editExpense(item: Expense) {
  editingExpense.value = item
  editingIncome.value = null
  transactionPreset.value = null
  incomeTransactionPreset.value = null
  exchangePreview.value = null
  transactionModalOpen.value = true
}
function editIncome(item: Income) {
  editingExpense.value = null
  editingIncome.value = item
  transactionPreset.value = null
  incomeTransactionPreset.value = null
  exchangePreview.value = null
  transactionModalOpen.value = true
}
async function registerVariableRecurring(item: RecurringExpense) {
  editingExpense.value = null
  editingIncome.value = null
  incomeTransactionPreset.value = null
  transactionPreset.value = item
  exchangePreview.value = null
  if (item.currency_code === 'USD') {
    try {
      exchangePreview.value = await transactionsApi.previewRecurringExpenseExchangeRate(item.id, month.value)
    } catch (error) {
      toast.add({ title: apiErrorMessage(error), description: '為替レートを取得できませんでした。金額を手動で入力してください。', color: 'warning' })
    }
  }
  transactionModalOpen.value = true
}
function registerVariableRecurringIncome(item: RecurringIncome) {
  editingExpense.value = null
  editingIncome.value = null
  transactionPreset.value = null
  incomeTransactionPreset.value = item
  exchangePreview.value = null
  transactionModalOpen.value = true
}
function closeTransaction() {
  transactionModalOpen.value = false
  editingExpense.value = null
  editingIncome.value = null
  transactionPreset.value = null
  incomeTransactionPreset.value = null
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
    <UDashboardPanel id="home">
      <template #header>
        <UDashboardNavbar title="ホーム">
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
            <UButton
              icon="i-lucide-plus"
              aria-label="記録する"
              @click="openTransaction"
            >
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
        <div
          v-if="loading"
          class="space-y-4"
        >
          <USkeleton class="h-28 w-full" />
          <USkeleton class="h-28 w-full" />
        </div>
        <div
          v-else
          class="space-y-6"
        >
          <section
            class="grid grid-cols-2 overflow-hidden rounded-lg border border-default bg-elevated sm:grid-cols-3 sm:divide-x sm:divide-default"
            :aria-label="`${monthLabel}の収支概要`"
          >
            <div class="p-3 sm:p-6">
              <p class="text-sm text-muted">
                収入
              </p>
              <p :class="['mt-1 text-base font-bold tabular-nums whitespace-nowrap sm:text-2xl', projectedIncomeTotal > 0 ? 'text-primary' : 'text-default']">
                {{ formatSignedCurrency(projectedIncomeTotal, 'positive') }}
              </p>
              <UBadge
                v-if="forecast.income"
                class="mt-2"
                color="neutral"
                variant="soft"
              >
                うち予定 {{ formatCurrency(forecast.income) }}
              </UBadge>
            </div>
            <div class="border-l border-default p-3 sm:border-t-0 sm:p-6">
              <p class="text-sm text-muted">
                支出
              </p>
              <p class="mt-1 text-base font-bold tabular-nums whitespace-nowrap sm:text-2xl">
                {{ formatSignedCurrency(projectedExpenseTotal, 'negative') }}
              </p>
              <UBadge
                v-if="forecast.expense"
                class="mt-2"
                color="neutral"
                variant="soft"
              >
                うち予定 {{ formatCurrency(forecast.expense) }}
              </UBadge>
            </div>
            <div class="col-span-2 border-t border-default p-3 sm:col-span-1 sm:border-t-0 sm:p-6">
              <p class="text-sm text-muted">
                収支
              </p>
              <p :class="['mt-1 text-lg font-bold tabular-nums whitespace-nowrap sm:text-2xl', balance < 0 ? 'text-error' : balance > 0 ? 'text-primary' : 'text-default']">
                {{ formatSignedCurrency(balance) }}
              </p>
              <UProgress
                v-if="budgetTotal"
                class="mt-2"
                :model-value="Math.min(100, budgetRate)"
                :color="budgetRate > 100 ? 'error' : budgetRate >= 80 ? 'warning' : 'primary'"
              />
              <p
                v-if="budgetTotal || forecast.expense || forecast.income"
                class="mt-2 text-xs text-muted"
              >
                {{ budgetTotal ? `予算 使用率 ${Math.round(budgetRate)}%` : '予定を含む' }}
              </p>
            </div>
          </section>

          <div class="grid gap-6 lg:grid-cols-[1.6fr_1fr]">
            <UCard>
              <template #header>
                <div class="flex items-center justify-between">
                  <h2 class="text-lg font-semibold">
                    明細
                  </h2>
                  <UBadge
                    color="neutral"
                    variant="outline"
                  >
                    {{ transactions.length }} 件
                  </UBadge>
                </div>
                <p class="text-sm text-muted">
                  {{ monthLabel }}
                </p>
              </template>
              <div v-if="transactionGroups.length">
                <section
                  v-for="group in transactionGroups"
                  :key="group.date"
                  class="border-b border-muted py-3 last:border-b-0"
                >
                  <div class="mb-1 flex items-center justify-between gap-3 text-xs text-muted">
                    <h3 class="font-medium text-toned">
                      {{ formatDate(group.date) }}
                    </h3>
                    <span class="tabular-nums">当日計 {{ formatSignedCurrency(group.total) }}</span>
                  </div>
                  <ul>
                    <li
                      v-for="item in group.items"
                      :key="item.id"
                      class="flex min-w-0 items-center gap-3 rounded-md py-2 hover:bg-elevated/50"
                    >
                      <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-semibold">
                          {{ transactionLabel(item) }}
                        </p>
                        <p class="truncate text-xs text-muted">
                          {{ transactionMeta(item) }}
                        </p>
                      </div>
                      <p :class="['shrink-0 font-semibold tabular-nums', item.kind === 'income' && Number(item.amount) > 0 ? 'text-primary' : 'text-default']">
                        {{ formatSignedCurrency(item.amount, item.kind === 'income' ? 'positive' : 'negative') }}
                      </p>
                      <UDropdownMenu :items="transactionActions(item)">
                        <UButton
                          icon="i-lucide-ellipsis"
                          color="neutral"
                          variant="ghost"
                          size="sm"
                          :aria-label="`${formatDate(item.transaction_date)} ${transactionLabel(item)}の操作`"
                        />
                      </UDropdownMenu>
                    </li>
                  </ul>
                </section>
              </div>
              <p
                v-else
                class="py-10 text-center text-sm text-muted"
              >
                この月の明細はまだありません。収入または支出を記録すると、ここに表示されます。
                <UButton
                  icon="i-lucide-plus"
                  class="mx-auto mt-4 flex w-fit"
                  @click="openTransaction"
                >
                  記録する
                </UButton>
              </p>
            </UCard>

            <div class="space-y-6">
              <UCard class="bg-default">
                <template #header>
                  <h2 class="text-lg font-semibold">
                    支出の内訳
                  </h2>
                </template>
                <div class="space-y-4">
                  <div
                    v-for="category in spending"
                    :key="category.id"
                  >
                    <div class="flex justify-between text-sm">
                      <b>{{ category.name }}</b>
                      <span class="tabular-nums">{{ formatCurrency(category.total) }} <small class="text-muted">{{ expenseTotal ? Math.round((category.total / expenseTotal) * 100) : 0 }}%</small></span>
                    </div>
                    <UProgress
                      class="mt-1"
                      :model-value="expenseTotal ? (category.total / expenseTotal) * 100 : 0"
                    />
                  </div>
                  <p
                    v-if="!spending.length"
                    class="text-sm text-muted"
                  >
                    この月の支出はまだありません。
                  </p>
                </div>
              </UCard>

              <UCard class="bg-default">
                <template #header>
                  <h2 class="text-lg font-semibold">
                    予算
                  </h2>
                </template>
                <div class="space-y-4">
                  <div
                    v-for="item in actuals"
                    :key="item.id"
                  >
                    <div class="flex justify-between gap-3 text-sm">
                      <b>{{ item.name }}</b>
                      <span :class="['tabular-nums', item.exceeded ? 'text-error' : item.achievementRate !== null && item.achievementRate >= 80 ? 'text-warning' : 'text-default']">{{ formatCurrency(item.actual) }} / {{ formatCurrency(item.budget) }}（{{ item.achievementRate === null ? '—' : `${Math.round(item.achievementRate)}%` }}）</span>
                    </div>
                    <UProgress
                      class="mt-1"
                      :model-value="item.achievementRate === null ? 0 : Math.min(100, item.achievementRate)"
                      :color="item.exceeded ? 'error' : item.achievementRate !== null && item.achievementRate >= 80 ? 'warning' : 'primary'"
                    />
                  </div>
                  <p
                    v-if="!actuals.length"
                    class="text-sm text-muted"
                  >
                    予算はまだ設定されていません。
                    <NuxtLink
                      to="/settings/budget"
                      class="font-medium text-primary hover:underline"
                    >予算を設定</NuxtLink>
                  </p>
                </div>
              </UCard>

              <UCard
                v-if="fixedRecurring.length || variableRecurring.length || fixedRecurringIncomes.length || variableRecurringIncomes.length"
                class="bg-default"
              >
                <template #header>
                  <h2 class="text-lg font-semibold">
                    定期の収支
                  </h2>
                </template>
                <ul class="divide-y divide-default">
                  <li
                    v-for="item in fixedRecurring"
                    :key="item.id"
                    class="flex items-center justify-between py-3 text-sm"
                  >
                    <span>
                      <b class="block">{{ item.name }}</b>
                      <small class="text-muted">毎月 {{ item.payment_day }} 日</small>
                    </span>
                    <b class="text-right">
                      <template v-if="item.currency_code === 'USD'">
                        <span
                          v-if="usdRecurringPreviews.get(item.id)"
                          class="block"
                        >
                          {{ formatCurrency(usdRecurringPreviews.get(item.id)!.converted_amount) }}
                        </span>
                        <span
                          v-else
                          class="font-normal"
                        >USD {{ item.foreign_amount ?? item.amount }}（換算待ち）</span>
                      </template>
                      <template v-else>
                        {{ formatCurrency(item.amount) }}
                      </template>
                    </b>
                  </li>
                </ul>
                <template v-if="variableRecurring.length">
                  <h3 class="mt-4 text-xs font-semibold tracking-widest text-muted uppercase">
                    準固定費（金額変動）
                  </h3>
                  <ul class="mt-2 divide-y divide-default">
                    <li
                      v-for="item in variableRecurring"
                      :key="item.id"
                      class="flex items-center justify-between py-3 text-sm"
                    >
                      <span>
                        <b class="block">{{ item.name }}</b>
                        <small class="text-muted">毎月 {{ item.payment_day }} 日 · 目安 {{ item.currency_code === 'USD' ? `USD ${item.foreign_amount ?? item.amount}` : formatCurrency(item.amount) }}</small>
                      </span>
                      <UButton
                        color="neutral"
                        variant="outline"
                        size="sm"
                        :aria-label="`${item.name}の今月分を登録`"
                        @click="registerVariableRecurring(item)"
                      >
                        今月分を登録
                      </UButton>
                    </li>
                  </ul>
                </template>
                <h3
                  v-if="fixedRecurringIncomes.length || variableRecurringIncomes.length"
                  class="mt-5 border-t border-default pt-5 text-sm font-semibold"
                >
                  収入
                </h3>
                <ul class="divide-y divide-default">
                  <li
                    v-for="item in fixedRecurringIncomes"
                    :key="item.id"
                    class="flex items-center justify-between py-3 text-sm"
                  >
                    <span>
                      <b class="block">{{ item.name }}</b>
                      <small class="text-muted">毎月 {{ item.payment_day }} 日</small>
                    </span>
                    <b>{{ formatCurrency(item.amount) }}</b>
                  </li>
                </ul>
                <template v-if="variableRecurringIncomes.length">
                  <h3 class="mt-4 text-xs font-semibold tracking-widest text-muted uppercase">
                    準固定収入（金額変動）
                  </h3>
                  <ul class="mt-2 divide-y divide-default">
                    <li
                      v-for="item in variableRecurringIncomes"
                      :key="item.id"
                      class="flex items-center justify-between py-3 text-sm"
                    >
                      <span>
                        <b class="block">{{ item.name }}</b>
                        <small class="text-muted">毎月 {{ item.payment_day }} 日 · 目安 {{ formatCurrency(item.amount) }}</small>
                      </span>
                      <UButton
                        color="neutral"
                        variant="outline"
                        size="sm"
                        :aria-label="`${item.name}の今月分を登録`"
                        @click="registerVariableRecurringIncome(item)"
                      >
                        今月分を登録
                      </UButton>
                    </li>
                  </ul>
                </template>
              </UCard>
              <UCard
                v-else
                class="bg-default"
              >
                <template #header>
                  <h2 class="text-lg font-semibold">
                    定期の収支
                  </h2>
                </template>
                <p class="mt-2 text-sm text-muted">
                  定期的な収支はまだ設定されていません。
                  <NuxtLink
                    to="/settings/recurring-expenses"
                    class="font-medium text-primary hover:underline"
                  >
                    定期の収支を設定
                  </NuxtLink>
                </p>
              </UCard>
            </div>
          </div>
        </div>
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
      :initial-recurring="transactionPreset"
      :initial-recurring-income="incomeTransactionPreset"
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
