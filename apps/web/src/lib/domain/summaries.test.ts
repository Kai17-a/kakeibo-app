import { describe, expect, it } from 'vitest';
import { dailyCategoryTotals, inPeriod, sumAmounts } from './summaries';

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
});
