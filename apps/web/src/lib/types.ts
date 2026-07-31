export interface BaseResource {
  id: string;
  created_at: string;
  updated_at: string;
}
export interface NamedResource extends BaseResource {
  name: string;
  description: string | null;
}
export type ExpenseCategory = NamedResource;
export type IncomeCategory = NamedResource;
export interface CategoryInput {
  name: string;
  description: string | null;
}
export type PaymentMethod = NamedResource;
export interface Expense extends BaseResource {
  transaction_date: string;
  amount: string;
  category_id: string;
  payment_method_id: string;
  recurring_expense_id: string | null;
  description: string | null;
}
export interface Income extends BaseResource {
  transaction_date: string;
  amount: string;
  category_id: string;
  description: string | null;
}
export interface RecurringExpense extends BaseResource {
  name: string;
  amount: string;
  payment_day: number;
  start_date: string;
  end_date: string | null;
  category_id: string;
  payment_method_id: string;
  is_active: boolean;
  is_variable: boolean;
  description: string | null;
}
export interface ListResponse<T> {
  items: T[];
  pagination: { page: number; per_page: number };
}
export interface ImportResult {
  imported: number;
  created_categories: string[];
  created_payment_methods: string[];
}
export interface ExpenseInput {
  transaction_date: string;
  amount: string;
  category_id: string;
  payment_method_id: string;
  recurring_expense_id: string | null;
  description: string | null;
}
export interface IncomeInput {
  transaction_date: string;
  amount: string;
  category_id: string;
  description: string | null;
}
export interface RecurringExpenseInput {
  name: string;
  amount: string;
  payment_day: number;
  start_date: string;
  end_date: string | null;
  category_id: string;
  payment_method_id: string;
  is_active: boolean;
  is_variable: boolean;
  description: string | null;
}
