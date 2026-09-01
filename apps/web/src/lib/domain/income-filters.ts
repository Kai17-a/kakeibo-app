import type { Income } from '../types';

export interface IncomeFilters {
  keyword: string;
  categoryId: string;
  paymentMethodId: string;
}

export function filterIncomes<
  T extends Pick<Income, 'description' | 'category_id' | 'payment_method_id'>,
>(incomes: T[], filters: IncomeFilters): T[] {
  const keyword = filters.keyword.trim().toLocaleLowerCase();

  return incomes.filter(
    (income) =>
      (!keyword || income.description?.toLocaleLowerCase().includes(keyword)) &&
      (!filters.categoryId || income.category_id === filters.categoryId) &&
      (!filters.paymentMethodId || income.payment_method_id === filters.paymentMethodId),
  );
}
