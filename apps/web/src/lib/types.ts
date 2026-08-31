export interface BaseResource {
  id: string;
  created_at: string;
  updated_at: string;
}
export interface NamedResource extends BaseResource {
  name: string;
  description: string | null;
}
export interface ExpenseCategory extends NamedResource {
  parent_category_id: string | null;
}
export interface IncomeCategory extends NamedResource {
  parent_category_id: string | null;
}
export interface CategoryInput {
  name: string;
  description: string | null;
  parent_category_id: string | null;
}
export interface PaymentMethodInput {
  name: string;
  description: string | null;
  initial_balance?: string;
}
export interface PaymentMethod extends NamedResource {
  initial_balance: string | null;
  balance: string | null;
}
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
  payment_method_id: string | null;
  recurring_income_id: string | null;
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
export interface RecurringIncome extends BaseResource {
  name: string;
  amount: string;
  payment_day: number;
  start_date: string;
  end_date: string | null;
  category_id: string;
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
export interface WebhookUrl extends BaseResource {
  url: string;
  description: string | null;
  is_active: boolean;
}
export interface WebhookUrlInput {
  url: string;
  description: string | null;
  is_active: boolean;
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
  payment_method_id?: string;
  recurring_income_id?: string | null;
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
export interface RecurringIncomeInput {
  name: string;
  amount: string;
  payment_day: number;
  start_date: string;
  end_date: string | null;
  category_id: string;
  is_active: boolean;
  is_variable: boolean;
  description: string | null;
}
