import { expect, test, type Page } from '@playwright/test'

const isoNow = new Date().toISOString()
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 }
const year = String(new Date().getFullYear())

const expenseCategory = { id: 'expense-category-1', name: '食費', description: null, parent_category_id: null, display_order: 0, created_at: isoNow, updated_at: isoNow }
const paymentMethod = { id: 'payment-method-1', name: '現金', description: null, initial_balance: '10000', balance: '10000', created_at: isoNow, updated_at: isoNow }
const expenses = [
  { id: 'expense-1', created_at: isoNow, updated_at: isoNow, transaction_date: `${year}-01-15`, amount: '5000', category_id: 'expense-category-1', payment_method_id: 'payment-method-1', recurring_expense_id: null, description: '食料品' },
  { id: 'expense-2', created_at: isoNow, updated_at: isoNow, transaction_date: `${year}-02-10`, amount: '3000', category_id: 'expense-category-1', payment_method_id: 'payment-method-1', recurring_expense_id: null, description: '外食' }
]
const incomes = [
  { id: 'income-1', created_at: isoNow, updated_at: isoNow, transaction_date: `${year}-01-25`, amount: '250000', category_id: 'income-category-1', payment_method_id: 'payment-method-1', recurring_income_id: null, description: '給与' }
]

async function mockAnnualApi(page: Page, overrides: Partial<Record<string, unknown>> = {}) {
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    const responses: Record<string, unknown> = {
      '/api/expenses': expenses,
      '/api/incomes': { items: incomes, pagination },
      '/api/expense-categories': { items: [expenseCategory], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': [],
      '/api/recurring-incomes': [],
      ...overrides
    }
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })
}

test('年間サマリー・内訳・テーブルが表示される', async ({ page }) => {
  await mockAnnualApi(page)
  await page.goto('/annual')

  await expect(page.getByText('年間収入', { exact: true })).toBeVisible()
  await expect(page.getByText(/250,000/).first()).toBeVisible()
  await expect(page.getByText('年間支出', { exact: true })).toBeVisible()
  await expect(page.getByText(/8,000/).first()).toBeVisible()

  await expect(page.getByRole('heading', { name: '年間支出の内訳' })).toBeVisible()
  await expect(page.getByText('食費').first()).toBeVisible()

  await expect(page.getByRole('heading', { name: '月ごとの収支' })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'カテゴリ別年間集計' })).toBeVisible()

  await expect(page.getByRole('heading', { name: '資産残高推移' })).toBeVisible()
  await expect(page.getByText('現金', { exact: true })).toBeVisible()
})

test('年選択を切り替えるとURLに反映される', async ({ page }) => {
  await mockAnnualApi(page)
  await page.goto(`/annual?year=${year}`)

  await page.getByRole('button', { name: '前年' }).click()
  await expect(page).toHaveURL(new RegExp(`/annual\\?year=${Number(year) - 1}$`))

  await page.getByRole('button', { name: '翌年' }).click()
  await page.getByRole('button', { name: '翌年' }).click()
  await expect(page).toHaveURL(new RegExp(`/annual\\?year=${Number(year) + 1}$`))
})

test('データがない年は空状態を表示する', async ({ page }) => {
  await mockAnnualApi(page, { '/api/expenses': [], '/api/incomes': { items: [], pagination } })
  await page.goto('/annual')

  await expect(page.getByText('この年の支出はまだありません。')).toBeVisible()
  await expect(page.getByText('この年の支出はありません。')).toBeVisible()
})
