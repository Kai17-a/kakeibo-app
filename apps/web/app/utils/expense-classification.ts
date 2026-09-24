import type { RecurringExpense } from "~/types/settings";
import type { Expense } from "~/types/transactions";

export interface MonthlyExpenseClassification {
  /** Active fixed recurring expenses applying to the month (counted at their planned amount). */
  fixed: RecurringExpense[];
  /** Expenses in categories of active variable recurring items (準固定費). */
  semiFixed: Expense[];
  /** All other expenses (変動費). */
  variable: Expense[];
}

function appliesTo(item: RecurringExpense, month: string) {
  return (
    item.is_active &&
    item.start_date.slice(0, 7) <= month &&
    (!item.end_date || item.end_date.slice(0, 7) >= month)
  );
}

/**
 * Splits a month's expenses into fixed, semi-fixed and variable costs. Expenses recorded for a
 * fixed recurring item are left out, because the item itself stands for them.
 */
export function classifyMonthlyExpenses(
  monthExpenses: Expense[],
  recurringExpenses: RecurringExpense[],
  month: string,
): MonthlyExpenseClassification {
  const fixed = recurringExpenses.filter((item) => !item.is_variable && appliesTo(item, month));
  const fixedIds = new Set(fixed.map((item) => item.id));
  const semiFixedCategoryIds = new Set(
    recurringExpenses
      .filter((item) => item.is_variable === true && appliesTo(item, month))
      .map((item) => item.category_id),
  );
  const unlinked = monthExpenses.filter(
    (item) => !(item.recurring_expense_id && fixedIds.has(item.recurring_expense_id)),
  );
  return {
    fixed,
    semiFixed: unlinked.filter((item) => semiFixedCategoryIds.has(item.category_id)),
    variable: unlinked.filter((item) => !semiFixedCategoryIds.has(item.category_id)),
  };
}
