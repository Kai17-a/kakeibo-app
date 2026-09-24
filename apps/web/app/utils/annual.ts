import type { Category, RecurringExpense, RecurringIncome } from "~/types/settings";
import type { Expense, Income } from "~/types/transactions";
import { annualMonthlyTotals, recurringForecast, sumAmounts } from "./summaries.ts";

/** The month key (YYYY-MM) of a 1-based month number in `year`. */
export function monthOfYear(year: string, month: number) {
  return `${year}-${String(month).padStart(2, "0")}`;
}

interface AnnualProjectionInput {
  year: string;
  expenses: Expense[];
  incomes: Income[];
  recurringExpenses: RecurringExpense[];
  recurringIncomes: RecurringIncome[];
  /** JPY conversions of USD recurring expenses, keyed by month and then recurring expense id. */
  usdPreviewsByMonth: Map<string, Map<string, { converted_amount: string }>>;
}

/**
 * Monthly totals for `year` where recurring items not yet recorded in a month are added at
 * their forecast amount.
 */
export function projectedAnnualMonths(input: AnnualProjectionInput) {
  return annualMonthlyTotals(input.expenses, input.incomes, input.year).map((item) => {
    const month = monthOfYear(input.year, item.month);
    const forecast = recurringForecast(
      input.expenses,
      input.incomes,
      input.recurringExpenses,
      input.recurringIncomes,
      month,
      input.usdPreviewsByMonth.get(month),
    );
    return {
      ...item,
      income: item.income + forecast.income,
      expense: item.expense + forecast.expense,
      balance: item.balance + forecast.income - forecast.expense,
      forecast,
    };
  });
}

export type ProjectedMonth = ReturnType<typeof projectedAnnualMonths>[number];

/** Per-category spending of each month (including forecasts); categories without any are left out. */
export function annualCategoryRows(
  year: string,
  expenses: Expense[],
  categories: Category[],
  months: ProjectedMonth[],
) {
  return categories
    .map((category) => {
      const values = months.map((item) => {
        const month = monthOfYear(year, item.month);
        return (
          sumAmounts(
            expenses.filter(
              (expense) =>
                expense.category_id === category.id && expense.transaction_date.startsWith(month),
            ),
          ) + (item.forecast.expensesByCategory.get(category.id) ?? 0)
        );
      });
      return {
        id: category.id,
        name: category.name,
        values,
        total: values.reduce((a, b) => a + b, 0),
      };
    })
    .filter((row) => row.total !== 0);
}

/**
 * The vertical scale of a bar chart whose values may be negative: `range` is the span from the
 * lowest to the highest value (at least 1), and `zeroOffset` is where zero sits, in percent from
 * the top.
 */
export function divergingScale(values: number[]) {
  const maxPositive = Math.max(0, ...values);
  const maxNegative = Math.max(0, ...values.map((value) => -value));
  const range = Math.max(1, maxPositive + maxNegative);
  return { range, zeroOffset: (maxPositive / range) * 100 };
}
