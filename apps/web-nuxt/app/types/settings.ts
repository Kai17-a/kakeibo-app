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
