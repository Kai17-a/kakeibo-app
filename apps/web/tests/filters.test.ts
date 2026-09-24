import assert from "node:assert/strict";
import { test } from "node:test";
import type { Expense, Income } from "../app/types/transactions.ts";
import { filterExpenses, filterIncomes } from "../app/utils/filters.ts";

const baseResource = { created_at: "2026-09-01T00:00:00Z", updated_at: "2026-09-01T00:00:00Z" };

const expenses: Expense[] = [
  {
    ...baseResource,
    id: "expense-1",
    transaction_date: "2026-09-01",
    amount: "1200",
    category_id: "food",
    payment_method_id: "cash",
    recurring_expense_id: null,
    description: "ランチ Café",
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    exchange_rate_date: null,
  },
  {
    ...baseResource,
    id: "expense-2",
    transaction_date: "2026-09-02",
    amount: "500",
    category_id: "transport",
    payment_method_id: "card",
    recurring_expense_id: null,
    description: null,
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
    exchange_rate_date: null,
  },
];

const incomes: Income[] = [
  {
    ...baseResource,
    id: "income-1",
    transaction_date: "2026-09-25",
    amount: "300000",
    category_id: "salary",
    payment_method_id: "bank",
    recurring_income_id: null,
    description: "September SALARY",
  },
  {
    ...baseResource,
    id: "income-2",
    transaction_date: "2026-09-30",
    amount: "10000",
    category_id: "bonus",
    payment_method_id: null,
    recurring_income_id: null,
    description: null,
  },
];

test("empty filters retain every item and empty inputs remain empty", () => {
  const filters = { keyword: "", categoryId: "", paymentMethodId: "" };

  assert.deepEqual(filterExpenses(expenses, filters), expenses);
  assert.deepEqual(filterIncomes(incomes, filters), incomes);
  assert.deepEqual(filterExpenses([], filters), []);
  assert.deepEqual(filterIncomes([], filters), []);
});

test("keyword matching trims whitespace and ignores locale-aware case", () => {
  const filters = { keyword: "  CAFÉ  ", categoryId: "", paymentMethodId: "" };

  assert.deepEqual(
    filterExpenses(expenses, filters).map((item) => item.id),
    ["expense-1"],
  );
  assert.deepEqual(
    filterIncomes(incomes, { ...filters, keyword: " salary " }).map((item) => item.id),
    ["income-1"],
  );
});

test("category and payment method filters must all match", () => {
  assert.deepEqual(
    filterExpenses(expenses, {
      keyword: "ランチ",
      categoryId: "food",
      paymentMethodId: "cash",
    }).map((item) => item.id),
    ["expense-1"],
  );
  assert.deepEqual(
    filterExpenses(expenses, { keyword: "ランチ", categoryId: "food", paymentMethodId: "card" }),
    [],
  );
  assert.deepEqual(
    filterIncomes(incomes, { keyword: "", categoryId: "salary", paymentMethodId: "bank" }).map(
      (item) => item.id,
    ),
    ["income-1"],
  );
});

test("missing descriptions do not match a non-empty keyword", () => {
  assert.deepEqual(
    filterExpenses(expenses, { keyword: "anything", categoryId: "", paymentMethodId: "" }),
    [],
  );
  assert.deepEqual(
    filterIncomes(incomes, { keyword: "anything", categoryId: "", paymentMethodId: "" }),
    [],
  );
});
