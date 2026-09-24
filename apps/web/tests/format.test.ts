import assert from "node:assert/strict";
import { test } from "node:test";
import { formatCurrency, formatSignedCurrency } from "../app/utils/format.ts";

test("formats zero without a sign regardless of direction", () => {
  assert.equal(formatSignedCurrency(0), formatCurrency(0));
  assert.equal(formatSignedCurrency("0", "positive"), formatCurrency(0));
  assert.equal(formatSignedCurrency(0, "negative"), formatCurrency(0));
});

test("derives the sign from the value in auto mode", () => {
  assert.equal(formatSignedCurrency(1500), `+${formatCurrency(1500)}`);
  assert.equal(formatSignedCurrency(-1500), `−${formatCurrency(1500)}`);
  assert.equal(formatSignedCurrency("-1234567"), `−${formatCurrency(1234567)}`);
});

test("uses the given direction for non-zero amounts", () => {
  assert.equal(formatSignedCurrency(800, "negative"), `−${formatCurrency(800)}`);
  assert.equal(formatSignedCurrency(-800, "positive"), `+${formatCurrency(800)}`);
});
