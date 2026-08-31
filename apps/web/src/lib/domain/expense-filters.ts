import type { Expense } from '../types';

export interface ExpenseFilters {
  keyword: string;
  categoryId: string;
  paymentMethodId: string;
}

export function filterExpenses<
  T extends Pick<Expense, 'description' | 'category_id' | 'payment_method_id'>,
>(expenses: T[], filters: ExpenseFilters): T[] {
  const keyword = filters.keyword.trim().toLocaleLowerCase();

  return expenses.filter(
    (expense) =>
      (!keyword || expense.description?.toLocaleLowerCase().includes(keyword)) &&
      (!filters.categoryId || expense.category_id === filters.categoryId) &&
      (!filters.paymentMethodId || expense.payment_method_id === filters.paymentMethodId),
  );
}
