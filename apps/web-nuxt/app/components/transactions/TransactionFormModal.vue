<script setup lang="ts">
import type { Category, PaymentMethod, RecurringExpense, RecurringIncome } from '~/types/settings'
import type { ExchangeRatePreview, Expense, ExpenseInput, Income, IncomeInput } from '~/types/transactions'
import { groupCategories } from '~/utils/settings'
import { formatCurrency } from '~/utils/format'

const props = defineProps<{
  open: boolean
  expenseCategories: Category[]
  incomeCategories: Category[]
  paymentMethods: PaymentMethod[]
  saving: boolean
  initialDate: string
  initialExpense?: Expense | null
  initialIncome?: Income | null
  initialRecurring?: RecurringExpense | null
  initialRecurringIncome?: RecurringIncome | null
  exchangePreview?: ExchangeRatePreview | null
  onSubmit: (kind: 'expense' | 'income', input: ExpenseInput | IncomeInput, keepOpen: boolean) => Promise<boolean>
}>()

const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const editing = computed(() => Boolean(props.initialExpense ?? props.initialIncome))
const preset = computed(() => Boolean(props.initialRecurring ?? props.initialRecurringIncome))
const kind = ref<'expense' | 'income'>(props.initialIncome || props.initialRecurringIncome ? 'income' : 'expense')

function recurringDate() {
  const recurring = props.initialRecurring ?? props.initialRecurringIncome
  if (!recurring) return undefined
  const [yearValue, monthValue] = props.initialDate.split('-').map(Number) as [number, number]
  const monthIndex = monthValue - 1
  const lastDay = new Date(yearValue, monthIndex + 1, 0).getDate()
  const day = Math.min(recurring.payment_day, lastDay)
  return `${yearValue}-${String(monthIndex + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`
}

const state = reactive({
  date: props.initialExpense?.transaction_date ?? props.initialIncome?.transaction_date ?? recurringDate() ?? props.initialDate,
  amount: String(
    props.initialExpense?.amount
    ?? props.initialIncome?.amount
    ?? props.exchangePreview?.converted_amount
    ?? props.initialRecurring?.amount
    ?? props.initialRecurringIncome?.amount
    ?? ''
  ),
  categoryId: props.initialExpense?.category_id ?? props.initialIncome?.category_id ?? props.initialRecurring?.category_id ?? props.initialRecurringIncome?.category_id ?? '',
  paymentMethodId: props.initialExpense?.payment_method_id ?? props.initialIncome?.payment_method_id ?? props.initialRecurring?.payment_method_id ?? (kind.value === 'expense' ? (props.paymentMethods[0]?.id ?? '') : ''),
  description: props.initialExpense?.description ?? props.initialIncome?.description ?? (props.initialRecurring ?? props.initialRecurringIncome)?.name ?? ''
})

const categories = computed(() => kind.value === 'expense' ? props.expenseCategories : props.incomeCategories)
const categoryOptions = computed(() => groupCategories(categories.value).map(category => ({
  label: category.parent_category_id === null ? category.name : `  ${category.name}`,
  value: category.id
})))
const paymentMethodOptions = computed(() => props.paymentMethods.map(method => ({ label: method.name, value: method.id })))

watch(kind, (next, previous) => {
  if (next === previous) return
  state.paymentMethodId = next === 'expense' ? (props.paymentMethods[0]?.id ?? '') : ''
  if (!categories.value.some(category => category.id === state.categoryId)) {
    state.categoryId = categories.value[0]?.id ?? ''
  }
})

if (!state.categoryId) {
  state.categoryId = categories.value[0]?.id ?? ''
}

const title = computed(() => {
  if (props.initialExpense) return '支出を編集'
  if (props.initialIncome) return '収入を編集'
  if (preset.value) return props.initialRecurringIncome ? '準固定収入を登録' : '準固定費を登録'
  return '収支を登録'
})

async function handleSubmit(event: SubmitEvent) {
  const keepOpen = (event.submitter as HTMLButtonElement | null)?.value === 'continue'
  await submit(keepOpen)
}

async function submit(keepOpen: boolean) {
  const input = kind.value === 'expense'
    ? {
      transaction_date: state.date,
      amount: state.amount,
      category_id: state.categoryId,
      payment_method_id: state.paymentMethodId,
      recurring_expense_id: props.initialExpense?.recurring_expense_id ?? props.initialRecurring?.id ?? null,
      description: state.description || null,
      ...(props.exchangePreview
        ? {
            foreign_amount: props.exchangePreview.foreign_amount,
            currency_code: props.exchangePreview.currency_code,
            exchange_rate: props.exchangePreview.exchange_rate,
            exchange_rate_date: props.exchangePreview.exchange_rate_date
          }
        : {})
    } satisfies ExpenseInput
    : {
      transaction_date: state.date,
      amount: state.amount,
      category_id: state.categoryId,
      payment_method_id: state.paymentMethodId || undefined,
      recurring_income_id: props.initialIncome?.recurring_income_id ?? props.initialRecurringIncome?.id ?? null,
      description: state.description || null
    } satisfies IncomeInput
  const saved = await props.onSubmit(kind.value, input, keepOpen)
  if (saved && keepOpen) {
    state.amount = ''
    state.description = ''
  }
}
</script>

<template>
  <UModal
    :open="open"
    :title="title"
    description="日付や金額、カテゴリを入力してください。"
    :dismissible="!saving"
    :close="!saving"
    :ui="{ footer: 'justify-end' }"
    @update:open="(value) => emit('update:open', value)"
  >
    <template #body>
      <UTabs
        v-if="!editing && !preset"
        v-model="kind"
        :items="[{ label: '支出', value: 'expense' }, { label: '収入', value: 'income' }]"
        class="mb-4"
      />
      <form
        id="transaction-form"
        class="flex flex-col gap-5"
        @submit.prevent="handleSubmit"
      >
        <UFormField
          label="日付"
          required
        >
          <UInput
            v-model="state.date"
            type="date"
            required
            class="w-full"
          />
        </UFormField>
        <UFormField label="金額">
          <UInput
            :model-value="state.amount"
            type="number"
            min="1"
            step="1"
            required
            class="w-full"
            @update:model-value="(value) => state.amount = String(value ?? '')"
          />
          <template
            v-if="preset && exchangePreview"
            #help
          >
            USD {{ exchangePreview.foreign_amount }} × レート {{ exchangePreview.exchange_rate }}（適用日: {{ exchangePreview.exchange_rate_date }}） = {{ formatCurrency(exchangePreview.converted_amount) }}
          </template>
        </UFormField>
        <UFormField label="カテゴリ">
          <USelect
            v-model="state.categoryId"
            :items="categoryOptions"
            class="w-full"
          />
        </UFormField>
        <UFormField :label="`支払方法${kind === 'income' ? '（任意）' : ''}`">
          <USelect
            v-model="state.paymentMethodId"
            :items="kind === 'income' ? [{ label: '未選択', value: '' }, ...paymentMethodOptions] : paymentMethodOptions"
            class="w-full"
          />
        </UFormField>
        <UFormField label="メモ（任意）">
          <UTextarea
            v-model="state.description"
            class="w-full"
          />
        </UFormField>
      </form>
    </template>
    <template #footer>
      <UButton
        color="neutral"
        variant="outline"
        :disabled="saving"
        @click="emit('update:open', false)"
      >
        キャンセル
      </UButton>
      <UButton
        v-if="!editing && !preset"
        type="submit"
        form="transaction-form"
        value="continue"
        color="neutral"
        variant="outline"
        :loading="saving"
        :disabled="saving"
      >
        登録して続ける
      </UButton>
      <UButton
        type="submit"
        form="transaction-form"
        :loading="saving"
        :disabled="saving"
      >
        {{ editing ? '更新する' : '登録する' }}
      </UButton>
    </template>
  </UModal>
</template>
