import type { Expense, ExpenseCategory, Income } from '../types';

export type Transaction = (Expense & { kind: 'expense' }) | (Income & { kind: 'income' });

export function inPeriod<T extends { transaction_date: string }>(items: T[], period: string): T[] {
  return items.filter((item) => item.transaction_date.startsWith(period));
}

export function sumAmounts(items: Array<{ amount: string }>): number {
  return items.reduce((sum, item) => sum + Number(item.amount), 0);
}

export function mergeTransactions(expenses: Expense[], incomes: Income[]): Transaction[] {
  return [
    ...expenses.map((item) => ({ ...item, kind: 'expense' as const })),
    ...incomes.map((item) => ({ ...item, kind: 'income' as const }))
  ].sort((a, b) => b.transaction_date.localeCompare(a.transaction_date));
}

export function categoryTotals(expenses: Expense[], categories: ExpenseCategory[]) {
  return categories.map((category) => ({
    id: category.id,
    name: category.name,
    total: sumAmounts(expenses.filter((item) => item.category_id === category.id))
  }));
}

export function dailyCategoryTotals(
  expenses: Array<Pick<Expense, 'transaction_date' | 'amount' | 'category_id'>>,
  month: string
) {
  const days = new Date(Number(month.slice(0, 4)), Number(month.slice(5, 7)), 0).getDate();
  return Array.from({ length: days }, (_, index) => {
    const date = `${month}-${String(index + 1).padStart(2, '0')}`;
    const values = new Map<string, number>();
    for (const item of expenses.filter((expense) => expense.transaction_date === date)) {
      values.set(item.category_id, (values.get(item.category_id) ?? 0) + Number(item.amount));
    }
    return { date, values, total: [...values.values()].reduce((sum, value) => sum + value, 0) };
  });
}

export function annualMonthlyTotals(expenses: Expense[], incomes: Income[], year: string) {
  return Array.from({ length: 12 }, (_, index) => {
    const month = `${year}-${String(index + 1).padStart(2, '0')}`;
    const income = sumAmounts(inPeriod(incomes, month));
    const expense = sumAmounts(inPeriod(expenses, month));
    return { month: index + 1, income, expense, balance: income - expense };
  });
}
