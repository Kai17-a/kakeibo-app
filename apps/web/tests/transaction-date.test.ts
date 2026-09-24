import assert from "node:assert/strict";
import { test } from "node:test";
import { defaultTransactionDate } from "../app/utils/transaction-date.ts";

test("uses today's day within the selected month", () => {
  assert.equal(defaultTransactionDate("2026-09", new Date(2026, 8, 12)), "2026-09-12");
});

test("keeps the first day of the month", () => {
  assert.equal(defaultTransactionDate("2026-02", new Date(2026, 8, 1)), "2026-02-01");
});

test("clamps days after the 28th to the 28th", () => {
  assert.equal(defaultTransactionDate("2026-02", new Date(2026, 0, 31)), "2026-02-28");
  assert.equal(defaultTransactionDate("2026-04", new Date(2026, 8, 29)), "2026-04-28");
});
