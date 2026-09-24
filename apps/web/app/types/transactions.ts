import type { BaseResource } from "./settings";

export interface Expense extends BaseResource {
  transaction_date: string;
  amount: string;
  category_id: string;
  payment_method_id: string;
  recurring_expense_id: string | null;
  description: string | null;
  foreign_amount: string | null;
  currency_code: string | null;
  exchange_rate: string | null;
  exchange_rate_date: string | null;
}
export interface ExpenseInput {
  transaction_date: string;
  amount: string;
  category_id: string;
  payment_method_id: string;
  recurring_expense_id: string | null;
  description: string | null;
  foreign_amount?: string | null;
  currency_code?: string | null;
  exchange_rate?: string | null;
  exchange_rate_date?: string | null;
}

export interface Income extends BaseResource {
  transaction_date: string;
  amount: string;
  category_id: string;
  payment_method_id: string | null;
  recurring_income_id: string | null;
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

export interface ExchangeRatePreview {
  foreign_amount: string;
  currency_code: string;
  exchange_rate: string;
  exchange_rate_date: string;
  converted_amount: string;
}

export interface ListResponse<T> {
  items: T[];
  pagination: { page: number; per_page: number };
}
