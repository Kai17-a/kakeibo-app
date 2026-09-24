import assert from "node:assert/strict";
import { test } from "node:test";
import {
  recurringIncomeForm,
  recurringIncomeInput,
  validateRecurringIncome,
} from "../app/utils/recurring-incomes.ts";

const categories = [{ id: "salary" }];
const valid = {
  name: " 給与 ",
  amount: "300000",
  payment_day: "25",
  start_date: "2026-09-01",
  end_date: "",
  category_id: "salary",
  is_active: true,
  is_variable: false,
  description: " ",
};

test("new forms default to active and local date; edits retain false flags", () => {
  const state = recurringIncomeForm(undefined, new Date(2026, 8, 23));
  assert.equal(state.start_date, "2026-09-23");
  assert.equal(state.is_active, true);
  assert.equal(state.is_variable, false);
  assert.equal(state.payment_day, "1");
  const input = recurringIncomeInput({ ...valid, is_active: false, is_variable: true });
  assert.deepEqual(recurringIncomeForm({ ...input, id: "a", created_at: "", updated_at: "" }), {
    ...valid,
    name: "給与",
    description: "",
    is_active: false,
    is_variable: true,
  });
});

test("validates required fields, positive whole yen, categories and payment day boundaries", () => {
  assert.deepEqual(validateRecurringIncome(valid, categories), []);
  for (const amount of ["", "0", "-1", "1.5", "Infinity"]) {
    assert.ok(
      validateRecurringIncome({ ...valid, amount }, categories).some((e) => e.name === "amount"),
    );
  }
  for (const payment_day of ["", "0", "32", "1.5"]) {
    assert.ok(
      validateRecurringIncome({ ...valid, payment_day }, categories).some(
        (e) => e.name === "payment_day",
      ),
    );
  }
  for (const payment_day of ["1", "31"])
    assert.deepEqual(validateRecurringIncome({ ...valid, payment_day }, categories), []);
  assert.ok(
    validateRecurringIncome({ ...valid, name: " " }, categories).some((e) => e.name === "name"),
  );
  assert.ok(validateRecurringIncome(valid, []).some((e) => e.name === "category_id"));
});

test("dates must exist and end on or after the start date", () => {
  for (const start_date of ["", "2026-02-29", "2026-13-01"]) {
    assert.ok(
      validateRecurringIncome({ ...valid, start_date }, categories).some(
        (e) => e.name === "start_date",
      ),
    );
  }
  for (const end_date of ["2026-08-31", "2026-09-31"]) {
    assert.ok(
      validateRecurringIncome({ ...valid, end_date }, categories).some(
        (e) => e.name === "end_date",
      ),
    );
  }
  for (const end_date of ["", valid.start_date, "2027-01-01"])
    assert.deepEqual(validateRecurringIncome({ ...valid, end_date }, categories), []);
  assert.deepEqual(validateRecurringIncome({ ...valid, start_date: "2028-02-29" }, categories), []);
});

test("payload normalizes strings, numeric day and optional fields, preserving flags", () => {
  assert.deepEqual(recurringIncomeInput({ ...valid, is_active: false, is_variable: true }), {
    name: "給与",
    amount: "300000",
    payment_day: 25,
    start_date: "2026-09-01",
    end_date: null,
    category_id: "salary",
    is_active: false,
    is_variable: true,
    description: null,
  });
});
