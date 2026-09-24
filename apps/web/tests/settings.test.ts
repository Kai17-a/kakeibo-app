import assert from "node:assert/strict";
import { test } from "node:test";
import {
  groupCategories,
  categoryMove,
  availableBudgetCategories,
  validateNamed,
  validAmount,
} from "../app/utils/settings.ts";

const categories = [
  { id: "b", parent_category_id: null, display_order: 2 },
  { id: "a2", parent_category_id: "a", display_order: 3 },
  { id: "a", parent_category_id: null, display_order: 0 },
  { id: "a1", parent_category_id: "a", display_order: 1 },
];
test("groups parents and children by display order without mutating input", () => {
  assert.deepEqual(
    groupCategories(categories).map((c) => c.id),
    ["a", "a1", "a2", "b"],
  );
  assert.equal(categories[0]?.id, "b");
});
test("moves only siblings, sends every sibling and keeps child relationships", () => {
  const moved = categoryMove(categories, "a2", -1)!;
  assert.deepEqual(moved.input, { parent_category_id: "a", category_ids: ["a2", "a1"] });
  assert.deepEqual(
    groupCategories(moved.items).map((c) => c.id),
    ["a", "a2", "a1", "b"],
  );
  assert.deepEqual(categoryMove(categories, "a", 1)?.input.category_ids, ["b", "a"]);
  assert.equal(categoryMove(categories, "a", -1), null);
  assert.equal(categoryMove(categories, "a2", 1), null);
  assert.equal(categoryMove(categories, "missing", 1), null);
  assert.equal(categories[1]?.display_order, 3);
});
test("budget choices exclude assigned categories but retain the edited category", () => {
  const budgets = categories.map((c) => ({ category_id: c.id }));
  assert.deepEqual(availableBudgetCategories(categories, budgets), []);
  assert.deepEqual(
    availableBudgetCategories(categories, budgets, "a").map((c) => c.id),
    ["a"],
  );
  assert.equal(availableBudgetCategories(categories, []).length, 4);
});
test("validates required names, length limits and nonnegative whole yen", () => {
  assert.equal(validateNamed({ name: "  ", description: "" }).length, 1);
  assert.equal(validateNamed({ name: "a".repeat(101), description: "x".repeat(501) }).length, 2);
  assert.deepEqual(validateNamed({ name: " 食費 ", description: "" }), []);
  for (const value of ["", "-1", "1.5", "NaN", "Infinity"]) assert.equal(validAmount(value), false);
  for (const value of ["0", 100, "1000"]) assert.equal(validAmount(value), true);
});
