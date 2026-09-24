export interface BaseResource {
  id: string
  created_at: string
  updated_at: string
}
export interface NamedResource extends BaseResource {
  name: string
  description: string | null
}
export type CategoryKind = 'expense' | 'income'
export interface Category extends NamedResource {
  parent_category_id: string | null
  display_order: number
}
export interface CategoryInput {
  name: string
  description: string | null
  parent_category_id: string | null
}
export interface CategoryReorderInput {
  parent_category_id: string | null
  category_ids: string[]
}
export interface PaymentMethod extends NamedResource {
  initial_balance: string | null
  balance: string | null
}
export interface PaymentMethodInput {
  name: string
  description: string | null
  initial_balance?: string
}
export interface Budget extends BaseResource {
  category_id: string
  amount: string
}
export interface BudgetInput {
  category_id: string
  amount: string
}

export interface RecurringIncomeInput {
  name: string
  amount: string
  payment_day: number
  start_date: string
  end_date: string | null
  category_id: string
  is_active: boolean
  is_variable: boolean
  description: string | null
}
export interface RecurringIncome extends BaseResource, RecurringIncomeInput {}

export interface RecurringExpenseInput {
  name: string
  amount: string
  payment_day: number
  start_date: string
  end_date: string | null
  category_id: string
  payment_method_id: string
  is_active: boolean
  is_variable: boolean
  description: string | null
  foreign_amount?: string | null
  currency_code?: string | null
  exchange_rate?: string | null
  sync_future_transactions?: boolean
}
export interface RecurringExpense extends BaseResource {
  name: string
  amount: string
  payment_day: number
  start_date: string
  end_date: string | null
  category_id: string
  payment_method_id: string
  is_active: boolean
  is_variable: boolean
  description: string | null
  foreign_amount: string | null
  currency_code: string | null
  exchange_rate: string | null
}
export interface PendingMonthsResponse {
  months: string[]
}
export interface BackfillResponse {
  created: string[]
  skipped: string[]
}
