import type {
  Expense,
  ExpenseCategory,
  ExpenseInput,
  Income,
  IncomeCategory,
  IncomeInput,
  ListResponse,
  PaymentMethod,
  RecurringExpense,
  RecurringExpenseInput,
} from './types';

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: init?.body ? { 'Content-Type': 'application/json', ...init.headers } : init?.headers,
  });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { message?: string } | null;
    throw new ApiError(response.status, body?.message ?? `API request failed (${response.status})`);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export const api = {
  expenses: () => request<Expense[]>('/api/expenses'),
  incomes: async () => {
    const items: Income[] = [];
    let page = 1;
    while (true) {
      const response = await request<ListResponse<Income>>(
        `/api/incomes?sort_by=transaction_date&sort_order=desc&per_page=100&page=${page}`,
      );
      items.push(...response.items);
      if (response.items.length < 100) return { items, pagination: response.pagination };
      page += 1;
    }
  },
  expenseCategories: () =>
    request<ListResponse<ExpenseCategory>>(
      '/api/expense-categories?sort_by=name&sort_order=asc&per_page=100',
    ),
  incomeCategories: () =>
    request<ListResponse<IncomeCategory>>(
      '/api/income-categories?sort_by=name&sort_order=asc&per_page=100',
    ),
  paymentMethods: () =>
    request<ListResponse<PaymentMethod>>(
      '/api/payment-methods?sort_by=name&sort_order=asc&per_page=100',
    ),
  recurringExpenses: () => request<RecurringExpense[]>('/api/recurring-expenses'),
  createExpense: (input: ExpenseInput) =>
    request<Expense>('/api/expenses', { method: 'POST', body: JSON.stringify(input) }),
  createIncome: (input: IncomeInput) =>
    request<Income>('/api/incomes', { method: 'POST', body: JSON.stringify(input) }),
  createRecurringExpense: (input: RecurringExpenseInput) =>
    request<RecurringExpense>('/api/recurring-expenses', {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  deleteExpense: (id: string) => request<void>(`/api/expenses/${id}`, { method: 'DELETE' }),
  deleteIncome: (id: string) => request<void>(`/api/incomes/${id}`, { method: 'DELETE' }),
};
