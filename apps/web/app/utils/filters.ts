import type { Expense, Income } from "~/types/transactions";

export interface TransactionFilters {
  keyword: string;
  categoryId: string;
  paymentMethodId: string;
}

export function filterExpenses<
  T extends Pick<Expense, "description" | "category_id" | "payment_method_id">,
>(expenses: T[], filters: TransactionFilters): T[] {
  const keyword = filters.keyword.trim().toLocaleLowerCase();
  return expenses.filter(
    (expense) =>
      (!keyword || expense.description?.toLocaleLowerCase().includes(keyword)) &&
      (!filters.categoryId || expense.category_id === filters.categoryId) &&
      (!filters.paymentMethodId || expense.payment_method_id === filters.paymentMethodId),
  );
}

export function filterIncomes<
  T extends Pick<Income, "description" | "category_id" | "payment_method_id">,
>(incomes: T[], filters: TransactionFilters): T[] {
  const keyword = filters.keyword.trim().toLocaleLowerCase();
  return incomes.filter(
    (income) =>
      (!keyword || income.description?.toLocaleLowerCase().includes(keyword)) &&
      (!filters.categoryId || income.category_id === filters.categoryId) &&
      (!filters.paymentMethodId || income.payment_method_id === filters.paymentMethodId),
  );
}
