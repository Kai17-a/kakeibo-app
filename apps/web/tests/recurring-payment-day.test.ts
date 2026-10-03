import assert from "node:assert/strict";
import { test } from "node:test";
import {
  formatRecurringPaymentDay,
  formatRecurringSchedule,
  recurringPaymentDayForm,
  recurringPaymentDayValue,
  setRecurringMonthEnd,
  updateRecurringPaymentDay,
} from "../app/utils/recurring-payment-day.ts";

test("formats day 31 as month end", () => {
  assert.equal(formatRecurringSchedule(31), "毎月末");
  assert.equal(formatRecurringSchedule(15), "毎月15日");
  assert.equal(formatRecurringSchedule(15, true), "毎月 15 日");
  assert.equal(formatRecurringPaymentDay("31"), "月末");
});

test("maps the month-end form state to payment day 31", () => {
  const state = recurringPaymentDayForm(15);
  setRecurringMonthEnd(state, true);
  assert.deepEqual(state, { payment_day: "31", is_month_end: true });
  assert.equal(recurringPaymentDayValue(state), 31);
});

test("opens day 31 as month end and treats direct input as month end", () => {
  const state = recurringPaymentDayForm(31);
  assert.equal(state.is_month_end, true);
  setRecurringMonthEnd(state, false);
  assert.deepEqual(state, { payment_day: "1", is_month_end: false });
  updateRecurringPaymentDay(state, 31);
  assert.deepEqual(state, { payment_day: "31", is_month_end: true });
});
