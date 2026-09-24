import type { RecurringExpense, RecurringExpenseInput } from '../types/settings'

export function recurringExpenseForm(item?: RecurringExpense, today = new Date()) {
  const usdBased = item?.currency_code === 'USD'
  return {
    name: item?.name ?? '',
    amount: usdBased ? '' : (item?.amount ?? ''),
    usdBased,
    foreignAmount: item?.foreign_amount ?? '',
    payment_day: String(item?.payment_day ?? 1),
    start_date: item?.start_date ?? `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`,
    end_date: item?.end_date ?? '',
    category_id: item?.category_id ?? '',
    payment_method_id: item?.payment_method_id ?? '',
    is_active: item?.is_active ?? true,
    is_variable: item?.is_variable ?? false,
    description: item?.description ?? '',
    sync_future_transactions: false
  }
}
type RecurringExpenseForm = ReturnType<typeof recurringExpenseForm>

function validDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value.startsWith('0000')) return false
  const date = new Date(`${value}T00:00:00Z`)
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value
}

export function validateRecurringExpense(
  state: RecurringExpenseForm,
  categories: { id: string }[],
  paymentMethods: { id: string }[]
) {
  const errors: { name: string, message: string }[] = []
  if (!state.name.trim()) errors.push({ name: 'name', message: '名称を入力してください。' })
  if (state.usdBased) {
    if (!Number.isFinite(Number(state.foreignAmount)) || Number(state.foreignAmount) <= 0) {
      errors.push({ name: 'foreignAmount', message: '毎月のUSD金額を0より大きい数値で入力してください。' })
    }
  } else if (!/^\d+$/.test(state.amount.trim()) || !Number.isSafeInteger(Number(state.amount)) || Number(state.amount) < 1) {
    errors.push({ name: 'amount', message: '金額を1以上の整数で入力してください。' })
  }
  const day = Number(state.payment_day)
  if (!Number.isInteger(day) || day < 1 || day > 31) errors.push({ name: 'payment_day', message: '支払日を1〜31の整数で入力してください。' })
  if (!categories.some(item => item.id === state.category_id)) errors.push({ name: 'category_id', message: '支出カテゴリを選択してください。' })
  if (!paymentMethods.some(item => item.id === state.payment_method_id)) errors.push({ name: 'payment_method_id', message: '支払方法を選択してください。' })
  if (!validDate(state.start_date)) errors.push({ name: 'start_date', message: '有効な開始日を入力してください。' })
  if (state.end_date && (!validDate(state.end_date) || state.end_date < state.start_date)) {
    errors.push({ name: 'end_date', message: '終了日は開始日以降の有効な日付を入力してください。' })
  }
  return errors
}

export function recurringExpenseInput(state: RecurringExpenseForm): RecurringExpenseInput {
  const usdBased = state.usdBased
  return {
    name: state.name.trim(),
    amount: usdBased ? String(state.foreignAmount).trim() : state.amount.trim(),
    payment_day: Number(state.payment_day),
    start_date: state.start_date,
    end_date: state.end_date || null,
    category_id: state.category_id,
    payment_method_id: state.payment_method_id,
    is_active: state.is_active,
    is_variable: state.is_variable,
    description: state.description.trim() || null,
    foreign_amount: usdBased && state.foreignAmount ? String(state.foreignAmount).trim() : null,
    currency_code: usdBased ? 'USD' : null,
    exchange_rate: null,
    sync_future_transactions: state.sync_future_transactions
  }
}

export function monthLabel(month: string): string {
  const match = month.match(/^(\d{4})-(\d{2})$/)
  if (!match) return `${month}分を登録する`
  const [, year, monthNumber] = match
  return `${year}年${Number(monthNumber)}月分を登録する`
}
