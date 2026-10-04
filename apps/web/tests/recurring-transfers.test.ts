import assert from "node:assert/strict";
import { test } from "node:test";
import {
  recurringTransferForm,
  recurringTransferInput,
  validateRecurringTransfer,
} from "../app/utils/recurring-transfers.ts";

const methods = [
  { id: "bank", name: "銀行" },
  { id: "nisa", name: "NISA" },
] as never[];
const valid = {
  name: " つみたてNISA ",
  amount: "30000",
  payment_day: "31",
  is_month_end: true,
  start_date: "2026-01-01",
  end_date: "",
  from_payment_method_id: "bank",
  to_payment_method_id: "nisa",
  is_active: true,
  description: " ",
};

test("form state and payload preserve the recurring transfer fields", () => {
  const input = recurringTransferInput(valid);
  assert.deepEqual(input, {
    name: "つみたてNISA",
    amount: "30000",
    payment_day: 31,
    start_date: "2026-01-01",
    end_date: null,
    from_payment_method_id: "bank",
    to_payment_method_id: "nisa",
    is_active: true,
    description: null,
  });
  assert.equal(recurringTransferForm(undefined, new Date(2026, 8, 23)).start_date, "2026-09-23");
});

test("validation covers amount, dates, payment day and distinct existing methods", () => {
  assert.deepEqual(validateRecurringTransfer(valid, methods), []);
  for (const amount of ["", "0", "-1", "1.5"])
    assert.ok(
      validateRecurringTransfer({ ...valid, amount }, methods).some(
        (error) => error.name === "amount",
      ),
    );
  assert.ok(
    validateRecurringTransfer({ ...valid, payment_day: "32" }, methods).some(
      (error) => error.name === "payment_day",
    ),
  );
  assert.ok(
    validateRecurringTransfer({ ...valid, to_payment_method_id: "bank" }, methods).some(
      (error) => error.name === "to_payment_method_id",
    ),
  );
  assert.ok(
    validateRecurringTransfer({ ...valid, end_date: "2025-12-31" }, methods).some(
      (error) => error.name === "end_date",
    ),
  );
  assert.ok(
    validateRecurringTransfer(valid, methods.slice(0, 1)).some(
      (error) => error.name === "to_payment_method_id",
    ),
  );
});
