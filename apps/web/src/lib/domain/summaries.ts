import type { Budget, Expense, ExpenseCategory, Income, PaymentMethod } from '../types';

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
    ...incomes.map((item) => ({ ...item, kind: 'income' as const })),
  ].sort((a, b) => b.transaction_date.localeCompare(a.transaction_date));
}

export function categoryTotals(expenses: Expense[], categories: ExpenseCategory[]) {
  return categories.map((category) => ({
    id: category.id,
    name: category.name,
    total: sumAmounts(expenses.filter((item) => item.category_id === category.id)),
  }));
}

export function dailyCategoryTotals(
  expenses: Array<Pick<Expense, 'transaction_date' | 'amount' | 'category_id'>>,
  month: string,
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

export function categoryMonthlyTotals(
  expenses: Expense[],
  categories: ExpenseCategory[],
  year: string,
) {
  const annualExpenses = inPeriod(expenses, year);
  const activeCategories = categories.filter(
    (category) =>
      sumAmounts(annualExpenses.filter((expense) => expense.category_id === category.id)) !== 0,
  );

  return Array.from({ length: 12 }, (_, index) => {
    const month = `${year}-${String(index + 1).padStart(2, '0')}`;
    const monthlyExpenses = inPeriod(annualExpenses, month);
    const values = activeCategories.map((category) => ({
      id: category.id,
      name: category.name,
      total: sumAmounts(monthlyExpenses.filter((expense) => expense.category_id === category.id)),
    }));
    return {
      month: index + 1,
      values,
      total: values.reduce((sum, value) => sum + value.total, 0),
    };
  });
}

export function budgetActuals(
  expenses: Expense[],
  categories: ExpenseCategory[],
  budgets: Budget[],
  month: string,
) {
  const monthlyExpenses = inPeriod(expenses, month);
  const categoryNames = new Map(categories.map((category) => [category.id, category.name]));

  return budgets.map((budget) => {
    const budgetAmount = Number(budget.amount);
    const actual = sumAmounts(
      monthlyExpenses.filter((expense) => expense.category_id === budget.category_id),
    );
    return {
      id: budget.id,
      categoryId: budget.category_id,
      name: categoryNames.get(budget.category_id) ?? '名称なし',
      budget: budgetAmount,
      actual,
      achievementRate: budgetAmount === 0 ? null : (actual / budgetAmount) * 100,
      exceeded: actual > budgetAmount,
    };
  });
}

export function paymentMethodBalanceTrend(
  incomes: Income[],
  expenses: Expense[],
  paymentMethods: PaymentMethod[],
  year: string,
) {
  const trackedMethods = paymentMethods.filter((method) => method.initial_balance !== null);
  const yearStart = `${year}-01`;
  const openingBalances = new Map(
    trackedMethods.map((method) => {
      const priorIncome = sumAmounts(
        incomes.filter(
          (income) => income.payment_method_id === method.id && income.transaction_date < yearStart,
        ),
      );
      const priorExpense = sumAmounts(
        expenses.filter(
          (expense) =>
            expense.payment_method_id === method.id && expense.transaction_date < yearStart,
        ),
      );
      return [method.id, Number(method.initial_balance) + priorIncome - priorExpense];
    }),
  );

  return Array.from({ length: 12 }, (_, index) => {
    const month = `${year}-${String(index + 1).padStart(2, '0')}`;
    const values = trackedMethods.map((method) => {
      const income = sumAmounts(
        incomes.filter(
          (item) => item.payment_method_id === method.id && item.transaction_date.startsWith(month),
        ),
      );
      const expense = sumAmounts(
        expenses.filter(
          (item) => item.payment_method_id === method.id && item.transaction_date.startsWith(month),
        ),
      );
      const balance = (openingBalances.get(method.id) ?? 0) + income - expense;
      openingBalances.set(method.id, balance);
      return { id: method.id, name: method.name, balance };
    });
    return {
      month: index + 1,
      values,
      total: values.reduce((sum, value) => sum + value.balance, 0),
    };
  });
}
