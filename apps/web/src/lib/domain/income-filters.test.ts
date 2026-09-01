import { describe, expect, it } from 'vitest';
import { filterIncomes, type IncomeFilters } from './income-filters';

describe('income filters', () => {
  const incomes = [
    {
      id: '1',
      description: 'Monthly SALARY payment',
      category_id: 'salary',
      payment_method_id: 'bank',
    },
    {
      id: '2',
      description: 'フリマ売上',
      category_id: 'extra',
      payment_method_id: 'cash',
    },
    {
      id: '3',
      description: null,
      category_id: 'extra',
      payment_method_id: null,
    },
  ];
  const defaults: IncomeFilters = { keyword: '', categoryId: '', paymentMethodId: '' };

  it('treats a trimmed empty keyword and empty selections as no filters, including an unassigned payment method', () => {
    expect(filterIncomes(incomes, { ...defaults, keyword: '   ' })).toEqual(incomes);
  });

  it('matches descriptions partially without regard to case', () => {
    expect(filterIncomes(incomes, { ...defaults, keyword: ' salary ' })).toEqual([incomes[0]]);
  });

  it('filters by category', () => {
    expect(filterIncomes(incomes, { ...defaults, categoryId: 'extra' })).toEqual([
      incomes[1],
      incomes[2],
    ]);
  });

  it('excludes an unassigned payment method when a specific payment method is selected', () => {
    expect(filterIncomes(incomes, { ...defaults, paymentMethodId: 'bank' })).not.toContain(
      incomes[2],
    );
  });

  it('includes only matching non-null payment method IDs', () => {
    expect(filterIncomes(incomes, { ...defaults, paymentMethodId: 'cash' })).toEqual([incomes[1]]);
  });

  it('filters by category and payment method', () => {
    expect(
      filterIncomes(incomes, { ...defaults, categoryId: 'extra', paymentMethodId: 'cash' }),
    ).toEqual([incomes[1]]);
  });

  it('combines all filters with AND and excludes an unassigned payment method', () => {
    expect(
      filterIncomes(incomes, {
        keyword: 'フリマ',
        categoryId: 'extra',
        paymentMethodId: 'bank',
      }),
    ).toEqual([]);
  });
});
