import { expect, test, type Page } from '@playwright/test'

const now = new Date()
const month = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 }
const isoNow = new Date().toISOString()

const expenseCategory = { id: 'expense-category-1', name: '交通費', description: null, parent_category_id: null, display_order: 0, created_at: isoNow, updated_at: isoNow }
const incomeCategory = { id: 'income-category-1', name: '給与', description: null, parent_category_id: null, display_order: 0, created_at: isoNow, updated_at: isoNow }
const paymentMethod = { id: 'payment-method-1', name: 'VISA', description: null, initial_balance: null, balance: null, created_at: isoNow, updated_at: isoNow }
const fixedRecurring = {
  id: 'recurring-fixed-1',
  created_at: isoNow,
  updated_at: isoNow,
  name: '家賃',
  amount: '80000',
  foreign_amount: null,
  currency_code: null,
  exchange_rate: null,
  payment_day: 1,
  start_date: '2026-01-01',
  end_date: null,
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  is_active: true,
  is_variable: false,
  description: null
}
const expense = {
  id: 'expense-1',
  created_at: isoNow,
  updated_at: isoNow,
  transaction_date: `${month}-02`,
  amount: '2000',
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  recurring_expense_id: null,
  description: '電車代'
}

function baseResponses(overrides: Partial<Record<string, unknown>> = {}) {
  return {
    '/api/expenses': [expense],
    '/api/incomes': { items: [], pagination },
    '/api/expense-categories': { items: [expenseCategory], pagination },
    '/api/income-categories': { items: [incomeCategory], pagination },
    '/api/payment-methods': { items: [paymentMethod], pagination },
    '/api/recurring-expenses': [fixedRecurring],
    ...overrides
  }
}

async function mockDailyApi(page: Page, overrides: Partial<Record<string, unknown>> = {}) {
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    const responses = baseResponses(overrides)
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })
}

test('4つのタブを切り替えられる', async ({ page }) => {
  await mockDailyApi(page)
  await page.goto('/daily')

  await expect(page.getByText('収支サマリー')).toBeVisible()
  await page.getByRole('tab', { name: '支出明細' }).click()
  await expect(page.getByRole('cell', { name: '電車代', exact: true })).toBeVisible()
  await page.getByRole('tab', { name: '収入明細' }).click()
  await expect(page.getByText('この月の収入明細はありません。')).toBeVisible()
  await page.getByRole('tab', { name: '月ごとのカテゴリ別支出' }).click()
  await expect(page.getByText('日ごとのカテゴリ別支出')).toBeVisible()
})

test('支出明細から支出を編集・削除する', async ({ page }) => {
  let expenses: unknown[] = [expense]
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    if (path === '/api/expenses/expense-1' && request.method() === 'PUT') {
      expenses = [{ ...expense, ...(request.postDataJSON() as Record<string, unknown>) }]
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(expenses[0]) })
      return
    }
    if (path === '/api/expenses/expense-1' && request.method() === 'DELETE') {
      expenses = []
      await route.fulfill({ status: 204 })
      return
    }
    const responses = baseResponses({ '/api/expenses': expenses })
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })

  await page.goto('/daily')
  await page.getByRole('tab', { name: '支出明細' }).click()
  await page.getByRole('button', { name: '電車代の操作' }).click()
  await page.getByRole('menuitem', { name: '編集' }).click()
  await expect(page.getByRole('heading', { name: '支出を編集' })).toBeVisible()
  await page.getByLabel('金額').fill('3333')
  await page.getByRole('button', { name: '更新する' }).click()
  await expect(page.getByText('支出を更新しました', { exact: true })).toBeVisible()
  await expect(page.getByRole('cell', { name: /[¥￥]3,333/ })).toBeVisible()

  await page.getByRole('button', { name: '電車代の操作' }).click()
  await page.getByRole('menuitem', { name: '削除' }).click()
  await page.getByRole('button', { name: '削除', exact: true }).click()
  await expect(page.getByText('明細を削除しました', { exact: true })).toBeVisible()
  await expect(page.getByText('この月の支出明細はありません。')).toBeVisible()
})

test('支出明細をキーワードで絞り込む', async ({ page }) => {
  const secondExpense = { ...expense, id: 'expense-2', description: '書籍代', category_id: 'expense-category-1' }
  await mockDailyApi(page, { '/api/expenses': [expense, secondExpense] })
  await page.goto('/daily')
  await page.getByRole('tab', { name: '支出明細' }).click()

  await expect(page.getByRole('cell', { name: '電車代', exact: true })).toBeVisible()
  await expect(page.getByRole('cell', { name: '書籍代', exact: true })).toBeVisible()

  await page.getByLabel('備考を検索').fill('書籍')
  await expect(page.getByRole('cell', { name: '書籍代', exact: true })).toBeVisible()
  await expect(page.getByRole('cell', { name: '電車代', exact: true })).not.toBeVisible()

  await page.getByRole('button', { name: '条件をクリア' }).click()
  await expect(page.getByRole('cell', { name: '電車代', exact: true })).toBeVisible()
})
