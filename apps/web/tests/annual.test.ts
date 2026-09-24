import assert from "node:assert/strict";
import { test } from "node:test";
import type { Category, RecurringExpense } from "../app/types/settings.ts";
import type { Expense } from "../app/types/transactions.ts";
import {
  annualCategoryRows,
  divergingScale,
  monthOfYear,
  projectedAnnualMonths,
} from "../app/utils/annual.ts";

const stamp = { created_at: "", updated_at: "" };

function expense(overrides: Partial<Expense>): Expense {
  return {
    id: "expense",
    ...stamp,
    transaction_date: "2026-01-10",
    amount: "1000",
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

const rent: RecurringExpense = {
  id: "rent",
  ...stamp,
  name: "家賃",
  amount: "80000",
  payment_day: 1,
  start_date: "2026-11-01",
  end_date: null,
  category_id: "housing",
  payment_method_id: "bank",
  is_active: true,
  is_variable: false,
  description: null,
  foreign_amount: null,
  currency_code: null,
  exchange_rate: null,
};

const categories: Category[] = [
  {
    id: "food",
    ...stamp,
    name: "食費",
    description: null,
    parent_category_id: null,
    display_order: 0,
  },
  {
    id: "housing",
    ...stamp,
    name: "住居費",
    description: null,
    parent_category_id: null,
    display_order: 1,
  },
  {
    id: "unused",
    ...stamp,
    name: "未使用",
    description: null,
    parent_category_id: null,
    display_order: 2,
  },
];

test("formats month keys", () => {
  assert.equal(monthOfYear("2026", 3), "2026-03");
  assert.equal(monthOfYear("2026", 12), "2026-12");
});

test("adds forecasts of unrecorded recurring items to each month", () => {
  const months = projectedAnnualMonths({
    year: "2026",
    expenses: [expense({})],
    incomes: [],
    recurringExpenses: [rent],
    recurringIncomes: [],
    usdPreviewsByMonth: new Map(),
  });

  assert.equal(months.length, 12);
  assert.deepEqual(
    months.map((item) => item.expense),
    [1000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80000, 80000],
  );
  assert.equal(months[10]!.balance, -80000);
});

test("builds category rows including forecasts and skips categories without spending", () => {
  const expenses = [expense({})];
  const months = projectedAnnualMonths({
    year: "2026",
    expenses,
    incomes: [],
    recurringExpenses: [rent],
    recurringIncomes: [],
    usdPreviewsByMonth: new Map(),
  });

  const rows = annualCategoryRows("2026", expenses, categories, months);

  assert.deepEqual(
    rows.map((row) => [row.id, row.total]),
    [
      ["food", 1000],
      ["housing", 160000],
    ],
  );
  assert.equal(rows[1]!.values[10], 80000);
});

test("computes a diverging scale around zero", () => {
  assert.deepEqual(divergingScale([300, -100, 50]), { range: 400, zeroOffset: 75 });
  assert.deepEqual(divergingScale([200, 100]), { range: 200, zeroOffset: 100 });
  assert.deepEqual(divergingScale([-50]), { range: 50, zeroOffset: 0 });
  assert.deepEqual(divergingScale([]), { range: 1, zeroOffset: 0 });
});
