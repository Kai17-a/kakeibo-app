import ky, { isHTTPError, type Options } from 'ky';
import type {
  CategoryInput,
  Expense,
  ExpenseCategory,
  ExpenseInput,
  Income,
  IncomeCategory,
  IncomeInput,
  ImportResult,
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

const client = ky.create({
  retry: 0,
  hooks: {
    beforeError: [
      ({ error }) => {
        if (!isHTTPError<{ message?: string }>(error)) return error;
        const message =
          typeof error.data === 'object' && error.data !== null ? error.data.message : undefined;
        return new ApiError(
          error.response.status,
          message ?? `API request failed (${error.response.status})`,
        );
      },
    ],
  },
});

async function request<T>(path: string, options?: Options): Promise<T> {
  const baseUrl = globalThis.location?.origin ?? 'http://localhost';
  const response = await client(new URL(path, baseUrl), options);
  if (response.status === 204) return undefined as T;
  return response.json<T>();
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
  createExpenseCategory: (input: CategoryInput) =>
    request<ExpenseCategory>('/api/expense-categories', { method: 'post', json: input }),
  createIncomeCategory: (input: CategoryInput) =>
    request<IncomeCategory>('/api/income-categories', { method: 'post', json: input }),
  paymentMethods: () =>
    request<ListResponse<PaymentMethod>>(
      '/api/payment-methods?sort_by=name&sort_order=asc&per_page=100',
    ),
  recurringExpenses: () => request<RecurringExpense[]>('/api/recurring-expenses'),
  createExpense: (input: ExpenseInput) =>
    request<Expense>('/api/expenses', { method: 'post', json: input }),
  updateExpense: (id: string, input: ExpenseInput) =>
    request<Expense>(`/api/expenses/${id}`, { method: 'put', json: input }),
  createIncome: (input: IncomeInput) =>
    request<Income>('/api/incomes', { method: 'post', json: input }),
  createRecurringExpense: (input: RecurringExpenseInput) =>
    request<RecurringExpense>('/api/recurring-expenses', {
      method: 'post',
      json: input,
    }),
  deleteExpense: (id: string) => request<void>(`/api/expenses/${id}`, { method: 'delete' }),
  deleteIncome: (id: string) => request<void>(`/api/incomes/${id}`, { method: 'delete' }),
  importExpenses: (csv: string) =>
    request<ImportResult>('/api/import/expenses', {
      method: 'post',
      headers: { 'content-type': 'text/csv' },
      body: csv,
    }),
  importIncomes: (csv: string) =>
    request<ImportResult>('/api/import/incomes', {
      method: 'post',
      headers: { 'content-type': 'text/csv' },
      body: csv,
    }),
};
