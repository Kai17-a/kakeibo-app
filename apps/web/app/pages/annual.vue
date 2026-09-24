<script setup lang="ts">
import type { Category, PaymentMethod, RecurringExpense, RecurringIncome } from '~/types/settings'
import type { ExchangeRatePreview, Expense, Income } from '~/types/transactions'
import { annualMonthlyTotals, categoryMonthlyTotals, paymentMethodBalanceTrend, recurringForecast, sumAmounts } from '~/utils/summaries'
import { formatCurrency } from '~/utils/format'

useSeoMeta({ title: '年間集計' })

const route = useRoute()
const currentYear = () => String(new Date().getFullYear())
const year = computed(() => (typeof route.query.year === 'string' && /^\d{4}$/.test(route.query.year) ? route.query.year : currentYear()))
function setYear(next: string) {
  navigateTo({ path: '/annual', query: { year: next } }, { replace: true })
}

const settingsApi = useSettingsApi()
const transactionsApi = useTransactionsApi()

const expenses = ref<Expense[]>([])
const incomes = ref<Income[]>([])
const expenseCategories = ref<Category[]>([])
const paymentMethods = ref<PaymentMethod[]>([])
const recurringExpenses = ref<RecurringExpense[]>([])
const recurringIncomes = ref<RecurringIncome[]>([])
const loading = ref(true)
const loadError = ref('')

async function loadAll() {
  loading.value = true
  loadError.value = ''
  try {
    const [expenseData, incomeData, expenseCategoryData, paymentData, recurringData, recurringIncomeData] = await Promise.all([
      transactionsApi.expenses(),
      transactionsApi.incomes(),
      settingsApi.categories('expense').list(),
      settingsApi.paymentMethods.list(),
      settingsApi.recurringExpenses.list(),
      settingsApi.recurringIncomes.list()
    ])
    expenses.value = expenseData
    incomes.value = incomeData
    expenseCategories.value = expenseCategoryData
    paymentMethods.value = paymentData
    recurringExpenses.value = recurringData
    recurringIncomes.value = recurringIncomeData
  } catch (error) {
    loadError.value = apiErrorMessage(error)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const availableYears = computed(() => {
  const years = new Set([...expenses.value, ...incomes.value].map(item => item.transaction_date.slice(0, 4)))
  years.add(year.value)
  return [...years].sort().reverse()
})

const usdPreviewsByMonth = ref(new Map<string, Map<string, ExchangeRatePreview>>())
let usdPreviewRequestId = 0
watch([year, recurringExpenses], async () => {
  const requestId = ++usdPreviewRequestId
  const targets = recurringExpenses.value.filter(item => item.is_active && !item.is_variable && item.currency_code === 'USD')
  if (!targets.length) {
    if (requestId === usdPreviewRequestId) usdPreviewsByMonth.value = new Map()
    return
  }
  const monthList = Array.from({ length: 12 }, (_, index) => `${year.value}-${String(index + 1).padStart(2, '0')}`)
  const results = await Promise.allSettled(
    monthList.flatMap(month => targets.map(async item => [month, item.id, await transactionsApi.previewRecurringExpenseExchangeRate(item.id, month)] as const))
  )
  if (requestId !== usdPreviewRequestId) return
  const grouped = new Map<string, Map<string, ExchangeRatePreview>>()
  for (const result of results) {
    if (result.status !== 'fulfilled') continue
    const [month, id, preview] = result.value
    if (!grouped.has(month)) grouped.set(month, new Map())
    grouped.get(month)!.set(id, preview)
  }
  usdPreviewsByMonth.value = grouped
}, { immediate: true })

const months = computed(() => annualMonthlyTotals(expenses.value, incomes.value, year.value))
const projectedMonths = computed(() => months.value.map((item) => {
  const month = `${year.value}-${String(item.month).padStart(2, '0')}`
  const forecast = recurringForecast(expenses.value, incomes.value, recurringExpenses.value, recurringIncomes.value, month, usdPreviewsByMonth.value.get(month))
  return {
    ...item,
    income: item.income + forecast.income,
    expense: item.expense + forecast.expense,
    balance: item.balance + forecast.income - forecast.expense,
    forecast
  }
}))
const expenseTotal = computed(() => projectedMonths.value.reduce((sum, item) => sum + item.expense, 0))
const incomeTotal = computed(() => projectedMonths.value.reduce((sum, item) => sum + item.income, 0))
const balance = computed(() => incomeTotal.value - expenseTotal.value)
const chartMax = computed(() => Math.max(1, ...projectedMonths.value.flatMap(item => [item.income, item.expense])))

const categoryRows = computed(() => expenseCategories.value.map((category) => {
  const values = projectedMonths.value.map((item) => {
    const month = `${year.value}-${String(item.month).padStart(2, '0')}`
    return sumAmounts(expenses.value.filter(expense => expense.category_id === category.id && expense.transaction_date.startsWith(month))) + (item.forecast.expensesByCategory.get(category.id) ?? 0)
  })
  return { id: category.id, name: category.name, values, total: values.reduce((a, b) => a + b, 0) }
}).filter(row => row.total !== 0))
const spending = computed(() => [...categoryRows.value].sort((a, b) => b.total - a.total))

const chartColors = ['bg-primary', 'bg-info', 'bg-error', 'bg-warning', 'bg-success', 'bg-neutral-500']

// カテゴリ別月次推移
const categoryMonths = computed(() => categoryMonthlyTotals(expenses.value, expenseCategories.value, year.value))
const categoryChartMax = computed(() => Math.max(1, ...categoryMonths.value.map(item => item.total)))
const visibleCategories = computed(() => categoryMonths.value[0]?.values ?? [])

// 資産残高推移
const balanceMonths = computed(() => paymentMethodBalanceTrend(incomes.value, expenses.value, paymentMethods.value, year.value))
const visibleMethods = computed(() => balanceMonths.value[0]?.values ?? [])
const maxPositive = computed(() => Math.max(0, ...balanceMonths.value.flatMap(item => item.values.map(value => value.balance))))
const maxNegative = computed(() => Math.max(0, ...balanceMonths.value.flatMap(item => item.values.map(value => -value.balance))))
const balanceChartRange = computed(() => Math.max(1, maxPositive.value + maxNegative.value))
const positiveHeight = computed(() => (maxPositive.value / balanceChartRange.value) * 100)
</script>

<template>
  <UDashboardPanel id="annual">
    <template #header>
      <UDashboardNavbar title="年間集計">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
      <UDashboardToolbar>
        <template #left>
          <UFieldGroup>
            <UButton
              color="neutral"
              variant="outline"
              :to="{ path: '/', query: { month: `${year}-01` } }"
            >
              月間
            </UButton>
            <UButton
              color="neutral"
              variant="outline"
              :to="{ path: '/daily', query: { month: `${year}-01` } }"
            >
              日別集計
            </UButton>
            <UButton
              color="neutral"
              variant="solid"
              to="/annual"
            >
              年間
            </UButton>
          </UFieldGroup>
        </template>
        <template #right>
          <UFieldGroup>
            <UButton
              icon="i-lucide-chevron-left"
              color="neutral"
              variant="outline"
              aria-label="前年"
              @click="setYear(String(Number(year) - 1))"
            />
            <USelect
              :model-value="year"
              :items="availableYears.map(item => ({ label: `${item}年`, value: item }))"
              aria-label="対象年"
              class="w-28"
              @update:model-value="(value) => setYear(String(value))"
            />
            <UButton
              icon="i-lucide-chevron-right"
              color="neutral"
              variant="outline"
              aria-label="翌年"
              @click="setYear(String(Number(year) + 1))"
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
      <div
        v-else
        class="space-y-6"
      >
        <section class="grid gap-4 md:grid-cols-3">
          <UCard>
            <p class="text-sm text-muted">
              年間収入
            </p>
            <p class="mt-1 text-2xl font-bold">
              {{ formatCurrency(incomeTotal) }}
            </p>
            <p class="mt-2 text-sm text-muted">
              月平均 {{ formatCurrency(incomeTotal / 12) }}
            </p>
          </UCard>
          <UCard>
            <p class="text-sm text-muted">
              年間支出
            </p>
            <p class="mt-1 text-2xl font-bold">
              {{ formatCurrency(expenseTotal) }}
            </p>
            <p class="mt-2 text-sm text-muted">
              月平均 {{ formatCurrency(expenseTotal / 12) }}
            </p>
          </UCard>
          <UCard>
            <p class="text-sm text-muted">
              年間収支
            </p>
            <p class="mt-1 text-2xl font-bold">
              {{ formatCurrency(balance) }}
            </p>
            <p class="mt-2 text-sm text-muted">
              貯蓄率 {{ incomeTotal ? Math.round((balance / incomeTotal) * 100) : 0 }}%
            </p>
          </UCard>
        </section>

        <div class="grid gap-6 lg:grid-cols-[1.6fr_1fr]">
          <UCard>
            <template #header>
              <h2 class="text-lg font-semibold">
                月別の収支推移
              </h2>
            </template>
            <div class="flex h-72 items-end gap-1 border-b border-default sm:gap-2">
              <div
                v-for="item in projectedMonths"
                :key="item.month"
                class="flex h-full min-w-0 flex-1 flex-col justify-end"
              >
                <div class="flex h-[calc(100%_-_2rem)] items-end justify-center gap-0.5 sm:gap-1">
                  <div
                    class="w-1.5 bg-primary sm:w-3"
                    :style="{ height: `${(item.income / chartMax) * 100}%` }"
                    :title="formatCurrency(item.income)"
                  />
                  <div
                    class="w-1.5 bg-error sm:w-3"
                    :style="{ height: `${(item.expense / chartMax) * 100}%` }"
                    :title="formatCurrency(item.expense)"
                  />
                </div>
                <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs">{{ item.month }}月</span>
              </div>
            </div>
            <div class="mt-4 flex gap-4 text-xs text-muted">
              <span class="flex items-center gap-1.5"><span class="size-2.5 bg-primary" />収入</span>
              <span class="flex items-center gap-1.5"><span class="size-2.5 bg-error" />支出</span>
            </div>
          </UCard>
          <UCard>
            <template #header>
              <h2 class="text-lg font-semibold">
                年間支出の内訳
              </h2>
            </template>
            <div class="space-y-4">
              <div
                v-for="item in spending.slice(0, 6)"
                :key="item.id"
                class="flex justify-between text-sm"
              >
                <b>{{ item.name }}</b>
                <span>{{ formatCurrency(item.total) }}</span>
              </div>
              <p
                v-if="!spending.length"
                class="text-sm text-muted"
              >
                この年の支出はまだありません。
              </p>
            </div>
          </UCard>
        </div>

        <div class="grid gap-6 xl:grid-cols-2">
          <UCard>
            <template #header>
              <h2 class="text-lg font-semibold">
                カテゴリ別月次推移
              </h2>
              <p class="text-sm text-muted">
                月ごとの支出構成
              </p>
            </template>
            <template v-if="visibleCategories.length">
              <div class="flex h-72 items-end gap-1 border-b border-default sm:gap-2">
                <div
                  v-for="item in categoryMonths"
                  :key="item.month"
                  class="flex h-full min-w-0 flex-1 flex-col justify-end"
                >
                  <div class="flex h-[calc(100%_-_2rem)] flex-col-reverse justify-start">
                    <div
                      v-for="(value, index) in item.values"
                      v-show="value.total"
                      :key="value.id"
                      :class="['w-full', chartColors[index % chartColors.length]]"
                      :style="{ height: `${(value.total / categoryChartMax) * 100}%` }"
                      :title="`${value.name}: ${formatCurrency(value.total)}`"
                    />
                  </div>
                  <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs">{{ item.month }}月</span>
                </div>
              </div>
              <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted">
                <span
                  v-for="(category, index) in visibleCategories"
                  :key="category.id"
                  class="flex items-center gap-1.5"
                >
                  <span :class="['size-2.5', chartColors[index % chartColors.length]]" />
                  {{ category.name }}
                </span>
              </div>
            </template>
            <p
              v-else
              class="py-24 text-center text-sm text-muted"
            >
              この年の支出はありません。
            </p>
          </UCard>

          <UCard>
            <template #header>
              <h2 class="text-lg font-semibold">
                資産残高推移
              </h2>
              <p class="text-sm text-muted">
                支払方法ごとの月末残高
              </p>
            </template>
            <template v-if="visibleMethods.length">
              <div class="relative flex h-72 gap-1 sm:gap-2">
                <div
                  v-for="item in balanceMonths"
                  :key="item.month"
                  class="flex h-full min-w-0 flex-1 flex-col"
                >
                  <div class="relative h-[calc(100%_-_2rem)]">
                    <div
                      class="pointer-events-none absolute inset-x-0 border-t border-default"
                      :style="{ top: `${positiveHeight}%` }"
                    />
                    <div
                      v-for="(value, index) in item.values"
                      :key="value.id"
                      :class="['absolute', chartColors[index % chartColors.length]]"
                      :style="{
                        left: `${(index / item.values.length) * 100}%`,
                        width: `${100 / item.values.length}%`,
                        top: value.balance >= 0 ? `${positiveHeight - (value.balance / balanceChartRange) * 100}%` : `${positiveHeight}%`,
                        height: `${(Math.abs(value.balance) / balanceChartRange) * 100}%`
                      }"
                      :title="`${value.name}: ${formatCurrency(value.balance)}`"
                    />
                  </div>
                  <span class="mt-2 block text-center text-[10px] whitespace-nowrap sm:text-xs">{{ item.month }}月</span>
                </div>
              </div>
              <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted">
                <span
                  v-for="(method, index) in visibleMethods"
                  :key="method.id"
                  class="flex items-center gap-1.5"
                >
                  <span :class="['size-2.5', chartColors[index % chartColors.length]]" />
                  {{ method.name }}
                </span>
              </div>
            </template>
            <p
              v-else
              class="py-24 text-center text-sm text-muted"
            >
              初期残高が設定された支払方法はありません。
            </p>
          </UCard>
        </div>

        <UCard>
          <template #header>
            <h2 class="text-lg font-semibold">
              月ごとの収支
            </h2>
          </template>
          <UTable
            :data="projectedMonths"
            :columns="[
              { accessorKey: 'month', header: '月' },
              { accessorKey: 'income', header: '収入', meta: { class: { th: 'text-right', td: 'text-right tabular-nums' } } },
              { accessorKey: 'expense', header: '支出', meta: { class: { th: 'text-right', td: 'text-right tabular-nums' } } },
              { accessorKey: 'balance', header: '収支', meta: { class: { th: 'text-right', td: 'text-right font-bold tabular-nums' } } }
            ]"
          >
            <template #month-cell="{ row }">
              {{ row.original.month }}月
            </template>
            <template #income-cell="{ row }">
              {{ formatCurrency(row.original.income) }}
            </template>
            <template #expense-cell="{ row }">
              {{ formatCurrency(row.original.expense) }}
            </template>
            <template #balance-cell="{ row }">
              {{ formatCurrency(row.original.balance) }}
            </template>
          </UTable>
        </UCard>

        <UCard>
          <template #header>
            <h2 class="text-lg font-semibold">
              カテゴリ別年間集計
            </h2>
            <p class="text-sm text-muted">
              未計上の固定定期支出を予測額として含みます。
            </p>
          </template>
          <div class="overflow-x-auto">
            <table class="w-full min-w-[900px] text-sm">
              <thead>
                <tr class="border-b border-default">
                  <th class="px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                    カテゴリ
                  </th>
                  <th
                    v-for="item in projectedMonths"
                    :key="item.month"
                    class="px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap"
                  >
                    {{ item.month }}月
                  </th>
                  <th class="px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase whitespace-nowrap">
                    年間合計
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="row in categoryRows"
                  :key="row.id"
                  class="border-t border-default"
                >
                  <th class="px-3 py-2 text-left whitespace-nowrap">
                    {{ row.name }}
                  </th>
                  <td
                    v-for="(value, index) in row.values"
                    :key="index"
                    class="px-3 py-2 text-right whitespace-nowrap"
                  >
                    {{ formatCurrency(value) }}
                  </td>
                  <td class="px-3 py-2 text-right font-bold whitespace-nowrap">
                    {{ formatCurrency(row.total) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
