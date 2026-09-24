import type { Budget, Category, PaymentMethod, RecurringExpense } from "../../app/types/settings";
import type { Expense, Income } from "../../app/types/transactions";

export const isoNow = new Date().toISOString();

const now = new Date();
export const currentYear = String(now.getFullYear());
export const currentMonth = `${currentYear}-${String(now.getMonth() + 1).padStart(2, "0")}`;

const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };

/** Wraps items in the paginated list shape returned by list endpoints. */
export function listOf<T>(items: T[]) {
  return { items, pagination };
}

const timestamps = { created_at: isoNow, updated_at: isoNow };

export function category(overrides: Partial<Category> = {}): Category {
  return {
    id: "expense-category-1",
    name: "食費",
    description: null,
    parent_category_id: null,
    display_order: 0,
    ...timestamps,
    ...overrides,
  };
}

export function incomeCategory(overrides: Partial<Category> = {}): Category {
  return category({ id: "income-category-1", name: "給与", ...overrides });
}

export function paymentMethod(overrides: Partial<PaymentMethod> = {}): PaymentMethod {
  return {
    id: "payment-method-1",
    name: "現金",
    description: null,
    initial_balance: null,
    balance: null,
    ...timestamps,
    ...overrides,
  };
}

export function budget(overrides: Partial<Budget> = {}): Budget {
  return {
    id: "budget-1",
    category_id: "expense-category-1",
    amount: "30000",
    ...timestamps,
    ...overrides,
  };
}

export function expense(overrides: Partial<Expense> = {}): Expense {
  return {
    id: "expense-1",
    transaction_date: `${currentMonth}-02`,
    amount: "2000",
    category_id: "expense-category-1",
    payment_method_id: "payment-method-1",
    recurring_expense_id: null,
    description: "電車代",
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    exchange_rate_date: null,
    ...timestamps,
    ...overrides,
  };
}

export function income(overrides: Partial<Income> = {}): Income {
  return {
    id: "income-1",
    transaction_date: `${currentMonth}-25`,
    amount: "250000",
    category_id: "income-category-1",
    payment_method_id: "payment-method-1",
    recurring_income_id: null,
    description: "給与",
    ...timestamps,
    ...overrides,
  };
}

export function recurringExpense(overrides: Partial<RecurringExpense> = {}): RecurringExpense {
  return {
    id: "recurring-1",
    name: "電気代",
    amount: "8000",
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    payment_day: 15,
    start_date: "2026-01-01",
    end_date: null,
    category_id: "expense-category-1",
    payment_method_id: "payment-method-1",
    is_active: true,
    is_variable: false,
    description: null,
    ...timestamps,
    ...overrides,
  };
}
