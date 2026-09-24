import assert from "node:assert/strict";
import { test } from "node:test";
import type { Transaction } from "../app/utils/summaries.ts";
import { groupTransactionsByDate } from "../app/utils/transactions.ts";

function transaction(
  id: string,
  kind: Transaction["kind"],
  date: string,
  amount: string,
): Transaction {
  const base = {
    id,
    created_at: "",
    updated_at: "",
    transaction_date: date,
    amount,
    category_id: "category",
    description: null,
  };
  return kind === "expense"
    ? {
        ...base,
        kind,
        payment_method_id: "payment",
        recurring_expense_id: null,
        foreign_amount: null,
        currency_code: null,
        exchange_rate: null,
        exchange_rate_date: null,
      }
    : { ...base, kind, payment_method_id: null, recurring_income_id: null };
}

test("groups transactions by date in input order with the net total", () => {
  const items = [
    transaction("a", "expense", "2026-09-20", "1200"),
    transaction("b", "income", "2026-09-20", "5000"),
    transaction("c", "expense", "2026-09-18", "300"),
    transaction("d", "expense", "2026-09-20", "800"),
  ];

  const groups = groupTransactionsByDate(items);

  assert.deepEqual(
    groups.map((group) => [group.date, group.items.map((item) => item.id), group.total]),
    [
      ["2026-09-20", ["a", "b", "d"], 3000],
      ["2026-09-18", ["c"], -300],
    ],
  );
});

test("returns no groups for no transactions", () => {
  assert.deepEqual(groupTransactionsByDate([]), []);
});
