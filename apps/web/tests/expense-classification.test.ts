import assert from "node:assert/strict";
import { test } from "node:test";
import type { RecurringExpense } from "../app/types/settings.ts";
import type { Expense } from "../app/types/transactions.ts";
import { classifyMonthlyExpenses } from "../app/utils/expense-classification.ts";

function recurring(overrides: Partial<RecurringExpense>): RecurringExpense {
  return {
    id: "recurring",
    created_at: "",
    updated_at: "",
    name: "定期",
    amount: "1000",
    payment_day: 1,
    start_date: "2026-01-01",
    end_date: null,
    category_id: "rent",
    payment_method_id: "bank",
    is_active: true,
    is_variable: false,
    description: null,
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    ...overrides,
  };
}

function expense(overrides: Partial<Expense>): Expense {
  return {
    id: "expense",
    created_at: "",
    updated_at: "",
    transaction_date: "2026-09-10",
    amount: "500",
    category_id: "food",
    payment_method_id: "cash",
    recurring_expense_id: null,
    description: null,
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    exchange_rate_date: null,
    ...overrides,
  };
}

const ids = (items: { id: string }[]) => items.map((item) => item.id);

test("classifies fixed, semi-fixed and variable expenses of the month", () => {
  const recurringExpenses = [
    recurring({ id: "rent" }),
    recurring({ id: "water", is_variable: true, category_id: "utility" }),
  ];
  const expenses = [
    expense({ id: "paid-rent", category_id: "rent", recurring_expense_id: "rent" }),
    expense({ id: "water-bill", category_id: "utility" }),
    expense({ id: "lunch" }),
  ];

  const result = classifyMonthlyExpenses(expenses, recurringExpenses, "2026-09");

  assert.deepEqual(ids(result.fixed), ["rent"]);
  assert.deepEqual(ids(result.semiFixed), ["water-bill"]);
  assert.deepEqual(ids(result.variable), ["lunch"]);
});

test("ignores recurring items that are inactive or outside the month", () => {
  const recurringExpenses = [
    recurring({ id: "inactive", is_active: false }),
    recurring({ id: "future", start_date: "2026-10-01" }),
    recurring({ id: "ended", end_date: "2026-08-31" }),
    recurring({ id: "ends-this-month", end_date: "2026-09-15" }),
    recurring({
      id: "old-variable",
      is_variable: true,
      category_id: "food",
      end_date: "2026-08-31",
    }),
  ];

  const result = classifyMonthlyExpenses([expense({ id: "lunch" })], recurringExpenses, "2026-09");

  assert.deepEqual(ids(result.fixed), ["ends-this-month"]);
  assert.deepEqual(ids(result.semiFixed), []);
  assert.deepEqual(ids(result.variable), ["lunch"]);
});

test("keeps expenses linked to a recurring item that no longer applies", () => {
  const result = classifyMonthlyExpenses(
    [expense({ id: "old-rent", recurring_expense_id: "ended" })],
    [recurring({ id: "ended", end_date: "2026-08-31" })],
    "2026-09",
  );

  assert.deepEqual(ids(result.variable), ["old-rent"]);
});
