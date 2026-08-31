import { describe, expect, it } from 'vitest';
import { filterExpenses, type ExpenseFilters } from './expense-filters';

describe('expense filters', () => {
  const expenses = [
    {
      id: '1',
      description: 'Weekend GROCERY shopping',
      category_id: 'food',
      payment_method_id: 'card',
    },
    {
      id: '2',
      description: '電車代',
      category_id: 'transport',
      payment_method_id: 'cash',
    },
    {
      id: '3',
      description: null,
      category_id: 'food',
      payment_method_id: 'cash',
    },
  ];
  const defaults: ExpenseFilters = { keyword: '', categoryId: '', paymentMethodId: '' };

  it('treats a trimmed empty keyword and empty selections as no filters', () => {
    expect(filterExpenses(expenses, { ...defaults, keyword: '   ' })).toEqual(expenses);
  });

  it('matches descriptions partially without regard to case', () => {
    expect(filterExpenses(expenses, { ...defaults, keyword: ' grocery ' })).toEqual([expenses[0]]);
  });

  it('filters by category', () => {
    expect(filterExpenses(expenses, { ...defaults, categoryId: 'food' })).toEqual([
      expenses[0],
      expenses[2],
    ]);
  });

  it('filters by payment method', () => {
    expect(filterExpenses(expenses, { ...defaults, paymentMethodId: 'cash' })).toEqual([
      expenses[1],
      expenses[2],
    ]);
  });

  it('filters by category and payment method', () => {
    expect(
      filterExpenses(expenses, { ...defaults, categoryId: 'food', paymentMethodId: 'cash' }),
    ).toEqual([expenses[2]]);
  });

  it('combines all filters with AND', () => {
    expect(
      filterExpenses(expenses, {
        keyword: 'weekend',
        categoryId: 'food',
        paymentMethodId: 'cash',
      }),
    ).toEqual([]);
  });
});
