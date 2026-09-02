import { describe, expect, it } from 'vitest';
import type { Budget, Expense, ExpenseCategory, Income, PaymentMethod } from '../types';
import {
  budgetActuals,
  categoryMonthlyTotals,
  dailyCategoryTotals,
  inPeriod,
  paymentMethodBalanceTrend,
  sumAmounts,
} from './summaries';

const resourceFields = {
  created_at: '2026-01-01T00:00:00Z',
  updated_at: '2026-01-01T00:00:00Z',
};

function expense(
  id: string,
  transaction_date: string,
  amount: string,
  category_id: string,
  payment_method_id = 'cash',
): Expense {
  return {
    ...resourceFields,
    id,
    transaction_date,
    amount,
    category_id,
    payment_method_id,
    recurring_expense_id: null,
    description: null,
  };
}

function income(
  id: string,
  transaction_date: string,
  amount: string,
  payment_method_id: string | null,
): Income {
  return {
    ...resourceFields,
    id,
    transaction_date,
    amount,
    category_id: 'salary',
    payment_method_id,
    recurring_income_id: null,
    description: null,
  };
}

function category(id: string, name: string): ExpenseCategory {
  return {
    ...resourceFields,
    id,
    name,
    description: null,
    parent_category_id: null,
  };
}

function paymentMethod(id: string, name: string, initial_balance: string | null): PaymentMethod {
  return {
    ...resourceFields,
    id,
    name,
    description: null,
    initial_balance,
    balance: null,
  };
}

describe('summary calculations', () => {
  const expenses = [
    { transaction_date: '2026-07-02', amount: '100', category_id: 'food' },
    { transaction_date: '2026-07-02', amount: '250', category_id: 'food' },
    { transaction_date: '2026-08-01', amount: '500', category_id: 'other' },
  ];

  it('filters items by month', () => expect(inPeriod(expenses, '2026-07')).toHaveLength(2));
  it('sums string amounts', () => expect(sumAmounts(expenses)).toBe(850));
  it('groups daily category amounts', () => {
    const days = dailyCategoryTotals(expenses, '2026-07');
    expect(days[1].values.get('food')).toBe(350);
  });

  it('builds 12 monthly category totals and excludes categories with a zero annual total', () => {
    const result = categoryMonthlyTotals(
      [
        expense('e1', '2026-01-02', '100', 'food'),
        expense('e2', '2026-01-20', '50', 'food'),
        expense('e3', '2026-02-01', '300', 'rent'),
        expense('e4', '2025-12-31', '999', 'other'),
      ],
      [category('food', '食費'), category('rent', '住居費'), category('other', 'その他')],
      '2026',
    );

    expect(result).toHaveLength(12);
    expect(result[0]).toEqual({
      month: 1,
      values: [
        { id: 'food', name: '食費', total: 150 },
        { id: 'rent', name: '住居費', total: 0 },
      ],
      total: 150,
    });
    expect(result[1].total).toBe(300);
    expect(result[11].values).toHaveLength(2);
  });

  it('calculates monthly budget actuals and handles a zero budget without division', () => {
    const budgets: Budget[] = [
      { ...resourceFields, id: 'b1', category_id: 'food', amount: '200' },
      { ...resourceFields, id: 'b2', category_id: 'other', amount: '0' },
    ];
    const result = budgetActuals(
      [
        expense('e1', '2026-07-02', '250', 'food'),
        expense('e2', '2026-08-02', '100', 'food'),
        expense('e3', '2026-07-03', '10', 'other'),
      ],
      [category('food', '食費')],
      budgets,
      '2026-07',
    );

    expect(result).toEqual([
      {
        id: 'b1',
        categoryId: 'food',
        name: '食費',
        budget: 200,
        actual: 250,
        achievementRate: 125,
        exceeded: true,
      },
      {
        id: 'b2',
        categoryId: 'other',
        name: '名称なし',
        budget: 0,
        actual: 10,
        achievementRate: null,
        exceeded: true,
      },
    ]);
  });

  it('carries all transactions before the selected year into payment method balances', () => {
    const result = paymentMethodBalanceTrend(
      [
        income('i1', '2025-12-20', '500', 'bank'),
        income('i2', '2026-01-10', '200', 'bank'),
        income('i3', '2026-02-10', '50', null),
      ],
      [
        expense('e1', '2024-06-01', '100', 'food', 'bank'),
        expense('e2', '2026-01-15', '1700', 'food', 'bank'),
        expense('e3', '2026-02-01', '20', 'food', 'cash'),
      ],
      [paymentMethod('bank', '銀行', '1000'), paymentMethod('cash', '現金', null)],
      '2026',
    );

    expect(result).toHaveLength(12);
    expect(result[0]).toEqual({
      month: 1,
      values: [{ id: 'bank', name: '銀行', balance: -100 }],
      total: -100,
    });
    expect(result[1].values[0].balance).toBe(-100);
    expect(result[11].total).toBe(-100);
  });
});
