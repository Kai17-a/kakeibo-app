import assert from "node:assert/strict";
import { test } from "node:test";
import {
  recurringExpenseForm,
  recurringExpenseInput,
  validateRecurringExpense,
} from "../app/utils/recurring-expenses.ts";

const categories = [{ id: "utilities" }];
const paymentMethods = [{ id: "card" }];
const valid = {
  name: " 電気代 ",
  amount: "8000",
  usdBased: false,
  foreignAmount: "",
  payment_day: "15",
  start_date: "2026-09-01",
  end_date: "",
  category_id: "utilities",
  payment_method_id: "card",
  is_active: true,
  is_variable: false,
  description: " ",
  sync_future_transactions: false,
};

test("variable JPY expense accepts an empty amount and maps it to null", () => {
  const state = { ...valid, amount: "", is_variable: true };
  assert.deepEqual(validateRecurringExpense(state, categories, paymentMethods), []);
  assert.equal(recurringExpenseInput(state).amount, null);
});

test("fixed JPY expense still requires a positive whole-yen amount", () => {
  for (const amount of ["", "0", "-1", "1.5"]) {
    assert.ok(
      validateRecurringExpense({ ...valid, amount }, categories, paymentMethods).some(
        (error) => error.name === "amount",
      ),
    );
  }
});

test("an existing variable estimate remains supported", () => {
  const state = recurringExpenseForm({
    ...recurringExpenseInput({ ...valid, is_variable: true }),
    id: "expense",
    created_at: "",
    updated_at: "",
    foreign_amount: null,
    currency_code: null,
    exchange_rate: null,
  });
  assert.equal(state.amount, "8000");
  assert.equal(recurringExpenseInput(state).amount, "8000");
});
