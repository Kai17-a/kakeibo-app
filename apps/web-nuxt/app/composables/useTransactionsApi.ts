import type { ExchangeRatePreview, Expense, ExpenseInput, Income, IncomeInput, ListResponse } from '~/types/transactions'

export function useTransactionsApi() {
  const client = $fetch.create({ retry: 0, timeout: 15000 })

  async function expenses() {
    return client<Expense[]>('/api/expenses')
  }

  async function incomes() {
    const items: Income[] = []
    let page = 1
    while (true) {
      const response = await client<ListResponse<Income>>('/api/incomes', {
        query: { sort_by: 'transaction_date', sort_order: 'desc', per_page: 100, page }
      })
      items.push(...response.items)
      if (response.items.length < 100) return items
      page += 1
    }
  }

  return {
    expenses,
    incomes,
    createExpense: (body: ExpenseInput) => client<Expense>('/api/expenses', { method: 'POST', body }),
    updateExpense: (id: string, body: ExpenseInput) => client<Expense>(`/api/expenses/${encodeURIComponent(id)}`, { method: 'PUT', body }),
    deleteExpense: async (id: string): Promise<void> => {
      await client(`/api/expenses/${encodeURIComponent(id)}`, { method: 'DELETE' })
    },
    createIncome: (body: IncomeInput) => client<Income>('/api/incomes', { method: 'POST', body }),
    updateIncome: (id: string, body: IncomeInput) => client<Income>(`/api/incomes/${encodeURIComponent(id)}`, { method: 'PUT', body }),
    deleteIncome: async (id: string): Promise<void> => {
      await client(`/api/incomes/${encodeURIComponent(id)}`, { method: 'DELETE' })
    },
    previewRecurringExpenseExchangeRate: (id: string, month: string) =>
      client<ExchangeRatePreview>(`/api/recurring-expenses/${encodeURIComponent(id)}/exchange-rate`, { query: { month } })
  }
}
