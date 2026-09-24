import assert from "node:assert/strict";
import { test } from "node:test";
import type {
  Budget,
  Category,
  PaymentMethod,
  RecurringExpense,
  RecurringIncome,
} from "../app/types/settings.ts";
import type { Expense, Income } from "../app/types/transactions.ts";
import {
  annualMonthlyTotals,
  budgetActuals,
  categoryMonthlyTotals,
  categoryTotals,
  dailyCategoryTotals,
  inPeriod,
  mergeTransactions,
  paymentMethodBalanceTrend,
  recurringForecast,
  sumAmounts,
} from "../app/utils/summaries.ts";

const timestamps = { created_at: "2026-01-01T00:00:00Z", updated_at: "2026-01-01T00:00:00Z" };

function expense(
  overrides: Partial<Expense> & Pick<Expense, "id" | "transaction_date" | "amount">,
): Expense {
  return {
    ...timestamps,
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

function income(
  overrides: Partial<Income> & Pick<Income, "id" | "transaction_date" | "amount">,
): Income {
  return {
    ...timestamps,
    category_id: "salary",
    payment_method_id: "bank",
    recurring_income_id: null,
    description: null,
    ...overrides,
  };
}

function category(id: string, name: string): Category {
  return { ...timestamps, id, name, description: null, parent_category_id: null, display_order: 0 };
}

function recurringExpense(
  overrides: Partial<RecurringExpense> & Pick<RecurringExpense, "id">,
): RecurringExpense {
  return {
    ...timestamps,
    name: "家賃",
    amount: "80000",
    payment_day: 27,
    start_date: "2026-01-01",
    end_date: null,
    category_id: "housing",
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

function recurringIncome(
  overrides: Partial<RecurringIncome> & Pick<RecurringIncome, "id">,
): RecurringIncome {
  return {
    ...timestamps,
    name: "給与",
    amount: "300000",
    payment_day: 25,
    start_date: "2026-01-01",
    end_date: null,
    category_id: "salary",
    is_active: true,
    is_variable: false,
    description: null,
    ...overrides,
  };
}

const expenses = [
  expense({ id: "e-aug", transaction_date: "2026-08-31", amount: "700" }),
  expense({ id: "e-start", transaction_date: "2026-09-01", amount: "1000" }),
  expense({ id: "e-end", transaction_date: "2026-09-30", amount: "2500", category_id: "fun" }),
  expense({ id: "e-oct", transaction_date: "2026-10-01", amount: "900" }),
];
const incomes = [
  income({ id: "i-start", transaction_date: "2026-09-01", amount: "10000" }),
  income({ id: "i-end", transaction_date: "2026-09-30", amount: "5000" }),
  income({ id: "i-next", transaction_date: "2027-01-01", amount: "999" }),
];

test("inPeriod uses the period prefix including both month boundaries", () => {
  assert.deepEqual(
    inPeriod(expenses, "2026-09").map((item) => item.id),
    ["e-start", "e-end"],
  );
  assert.deepEqual(
    inPeriod(expenses, "2026").map((item) => item.id),
    ["e-aug", "e-start", "e-end", "e-oct"],
  );
  assert.deepEqual(inPeriod([], "2026-09"), []);
});

test("sumAmounts converts decimal strings and returns zero for an empty array", () => {
  assert.equal(sumAmounts([{ amount: "100.5" }, { amount: "-20" }, { amount: "0" }]), 80.5);
  assert.equal(sumAmounts([]), 0);
});

test("mergeTransactions returns newest dates first without mutating inputs", () => {
  const expenseOrder = expenses.map((item) => item.id);
  const incomeOrder = incomes.map((item) => item.id);
  const merged = mergeTransactions(expenses, incomes);

  assert.deepEqual(
    merged.map((item) => `${item.kind}:${item.id}`),
    [
      "income:i-next",
      "expense:e-oct",
      "expense:e-end",
      "income:i-end",
      "expense:e-start",
      "income:i-start",
      "expense:e-aug",
    ],
  );
  assert.deepEqual(
    expenses.map((item) => item.id),
    expenseOrder,
  );
  assert.deepEqual(
    incomes.map((item) => item.id),
    incomeOrder,
  );
});

test("recurringForecast excludes posted, inactive, variable and out-of-period items", () => {
  const recurringExpenses = [
    recurringExpense({ id: "rent" }),
    recurringExpense({ id: "posted-expense", amount: "2000" }),
    recurringExpense({ id: "inactive-expense", amount: "3000", is_active: false }),
    recurringExpense({ id: "variable-expense", amount: "4000", is_variable: true }),
    recurringExpense({ id: "future-expense", amount: "5000", start_date: "2026-10-01" }),
    recurringExpense({ id: "ended-expense", amount: "6000", end_date: "2026-08-31" }),
    recurringExpense({ id: "starts-at-end", amount: "7000", start_date: "2026-09-30" }),
    recurringExpense({ id: "ends-at-start", amount: "8000", end_date: "2026-09-01" }),
  ];
  const recurringIncomes = [
    recurringIncome({ id: "salary" }),
    recurringIncome({ id: "posted-income", amount: "20000" }),
    recurringIncome({ id: "inactive-income", amount: "30000", is_active: false }),
    recurringIncome({ id: "variable-income", amount: "40000", is_variable: true }),
  ];
  const postedExpenses = [
    expense({
      id: "posted-e",
      transaction_date: "2026-09-01",
      amount: "2000",
      recurring_expense_id: "posted-expense",
    }),
  ];
  const postedIncomes = [
    income({
      id: "posted-i",
      transaction_date: "2026-09-30",
      amount: "20000",
      recurring_income_id: "posted-income",
    }),
  ];

  const result = recurringForecast(
    postedExpenses,
    postedIncomes,
    recurringExpenses,
    recurringIncomes,
    "2026-09",
  );

  assert.equal(result.expense, 95000);
  assert.equal(result.income, 300000);
  assert.deepEqual([...result.expensesByCategory], [["housing", 95000]]);
});

test("recurringForecast uses USD previews and skips USD items without one", () => {
  const result = recurringForecast(
    [],
    [],
    [
      recurringExpense({ id: "usd", amount: "999", currency_code: "USD", category_id: "travel" }),
      recurringExpense({ id: "usd-missing", amount: "888", currency_code: "USD" }),
      recurringExpense({ id: "yen", amount: "1200", currency_code: "JPY", category_id: "food" }),
    ],
    [],
    "2026-09",
    new Map([["usd", { converted_amount: "1500" }]]),
  );

  assert.equal(result.expense, 2700);
  assert.deepEqual(
    [...result.expensesByCategory],
    [
      ["travel", 1500],
      ["food", 1200],
    ],
  );
  assert.deepEqual(recurringForecast([], [], [], [], "2026-09"), {
    expense: 0,
    income: 0,
    expensesByCategory: new Map(),
  });
});

test("categoryTotals preserves category order and includes zero totals", () => {
  assert.deepEqual(
    categoryTotals(expenses, [category("fun", "娯楽"), category("none", "未使用")]),
    [
      { id: "fun", name: "娯楽", total: 2500 },
      { id: "none", name: "未使用", total: 0 },
    ],
  );
  assert.deepEqual(categoryTotals([], []), []);
});

test("annualMonthlyTotals returns all 12 numeric months with income, expense and balance", () => {
  const result = annualMonthlyTotals(expenses, incomes, "2026");

  assert.equal(result.length, 12);
  assert.deepEqual(result[0], { month: 1, income: 0, expense: 0, balance: 0 });
  assert.deepEqual(result[7], { month: 8, income: 0, expense: 700, balance: -700 });
  assert.deepEqual(result[8], { month: 9, income: 15000, expense: 3500, balance: 11500 });
  assert.deepEqual(result[9], { month: 10, income: 0, expense: 900, balance: -900 });
  assert.deepEqual(result[11], { month: 12, income: 0, expense: 0, balance: 0 });
  assert.deepEqual(
    annualMonthlyTotals([], [], "2026").map((item) => item.month),
    [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
  );
});

test("categoryMonthlyTotals keeps only categories active in the selected year", () => {
  const result = categoryMonthlyTotals(
    expenses,
    [category("food", "食費"), category("fun", "娯楽"), category("none", "未使用")],
    "2026",
  );

  assert.equal(result.length, 12);
  assert.deepEqual(result[8], {
    month: 9,
    values: [
      { id: "food", name: "食費", total: 1000 },
      { id: "fun", name: "娯楽", total: 2500 },
    ],
    total: 3500,
  });
  assert.deepEqual(
    result[0]?.values.map((item) => item.id),
    ["food", "fun"],
  );
  assert.ok(result.every((item) => item.values.every((value) => value.id !== "none")));
  assert.deepEqual(categoryMonthlyTotals([], [], "2026")[0], { month: 1, values: [], total: 0 });
});

test("paymentMethodBalanceTrend applies prior activity then carries monthly balances forward", () => {
  const methods: PaymentMethod[] = [
    {
      ...timestamps,
      id: "bank",
      name: "銀行",
      description: null,
      initial_balance: "10000",
      balance: null,
    },
    {
      ...timestamps,
      id: "cash",
      name: "現金",
      description: null,
      initial_balance: "500",
      balance: null,
    },
    {
      ...timestamps,
      id: "untracked",
      name: "未追跡",
      description: null,
      initial_balance: null,
      balance: null,
    },
  ];
  const result = paymentMethodBalanceTrend(
    [
      income({ id: "prior-income", transaction_date: "2025-12-31", amount: "2000" }),
      income({ id: "jan-income", transaction_date: "2026-01-31", amount: "3000" }),
    ],
    [
      expense({
        id: "prior-expense",
        transaction_date: "2025-12-01",
        amount: "1000",
        payment_method_id: "bank",
      }),
      expense({
        id: "jan-expense",
        transaction_date: "2026-01-01",
        amount: "400",
        payment_method_id: "bank",
      }),
      expense({
        id: "feb-cash",
        transaction_date: "2026-02-01",
        amount: "100",
        payment_method_id: "cash",
      }),
    ],
    methods,
    "2026",
  );

  assert.deepEqual(result[0], {
    month: 1,
    values: [
      { id: "bank", name: "銀行", balance: 13600 },
      { id: "cash", name: "現金", balance: 500 },
    ],
    total: 14100,
  });
  assert.deepEqual(result[1], {
    month: 2,
    values: [
      { id: "bank", name: "銀行", balance: 13600 },
      { id: "cash", name: "現金", balance: 400 },
    ],
    total: 14000,
  });
  assert.deepEqual(paymentMethodBalanceTrend([], [], [], "2026")[0], {
    month: 1,
    values: [],
    total: 0,
  });
});

test("dailyCategoryTotals formats every calendar date and aggregates categories per day", () => {
  const result = dailyCategoryTotals(
    [
      { transaction_date: "2024-02-01", amount: "100", category_id: "food" },
      { transaction_date: "2024-02-01", amount: "50", category_id: "food" },
      { transaction_date: "2024-02-29", amount: "200", category_id: "fun" },
      { transaction_date: "2024-03-01", amount: "999", category_id: "food" },
    ],
    "2024-02",
  );

  assert.equal(result.length, 29);
  assert.deepEqual(result[0], { date: "2024-02-01", values: new Map([["food", 150]]), total: 150 });
  assert.deepEqual(result[1], { date: "2024-02-02", values: new Map(), total: 0 });
  assert.deepEqual(result[28], { date: "2024-02-29", values: new Map([["fun", 200]]), total: 200 });
  assert.equal(dailyCategoryTotals([], "2026-04").length, 30);
});

test("budgetActuals reports actuals, zero-budget rates, exceeded state and missing names", () => {
  const budgets: Budget[] = [
    { ...timestamps, id: "budget-food", category_id: "food", amount: "3000" },
    { ...timestamps, id: "budget-fun", category_id: "fun", amount: "0" },
    { ...timestamps, id: "budget-missing", category_id: "missing", amount: "100" },
  ];

  assert.deepEqual(
    budgetActuals(
      expenses,
      [category("food", "食費"), category("fun", "娯楽")],
      budgets,
      "2026-09",
    ),
    [
      {
        id: "budget-food",
        categoryId: "food",
        name: "食費",
        budget: 3000,
        actual: 1000,
        achievementRate: (1000 / 3000) * 100,
        exceeded: false,
      },
      {
        id: "budget-fun",
        categoryId: "fun",
        name: "娯楽",
        budget: 0,
        actual: 2500,
        achievementRate: null,
        exceeded: true,
      },
      {
        id: "budget-missing",
        categoryId: "missing",
        name: "名称なし",
        budget: 100,
        actual: 0,
        achievementRate: 0,
        exceeded: false,
      },
    ],
  );
  assert.deepEqual(budgetActuals([], [], [], "2026-09"), []);
});
