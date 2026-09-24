import { expect, test, type Page } from '@playwright/test'

const now = new Date()
const month = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 }
const isoNow = new Date().toISOString()

const expenseCategory = { id: 'expense-category-1', name: '食費', description: null, parent_category_id: null, display_order: 0, created_at: isoNow, updated_at: isoNow }
const incomeCategory = { id: 'income-category-1', name: '給与', description: null, parent_category_id: null, display_order: 0, created_at: isoNow, updated_at: isoNow }
const paymentMethod = { id: 'payment-method-1', name: '現金', description: null, initial_balance: null, balance: null, created_at: isoNow, updated_at: isoNow }
const budget = { id: 'budget-1', category_id: 'expense-category-1', amount: '30000', created_at: isoNow, updated_at: isoNow }
const variableRecurring = {
  id: 'recurring-1',
  created_at: isoNow,
  updated_at: isoNow,
  name: '電気代',
  amount: '8000',
  foreign_amount: null,
  currency_code: null,
  exchange_rate: null,
  payment_day: 15,
  start_date: '2026-01-01',
  end_date: null,
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  is_active: true,
  is_variable: true,
  description: null
}
const usdVariableRecurring = { ...variableRecurring, id: 'recurring-usd-1', name: '動画サービス', amount: '10', foreign_amount: '10', currency_code: 'USD' }

function baseResponses(overrides: Partial<Record<string, unknown>> = {}) {
  return {
    '/api/expenses': [],
    '/api/incomes': { items: [], pagination },
    '/api/expense-categories': { items: [expenseCategory], pagination },
    '/api/income-categories': { items: [incomeCategory], pagination },
    '/api/payment-methods': { items: [paymentMethod], pagination },
    '/api/recurring-expenses': [variableRecurring],
    '/api/recurring-incomes': [],
    '/api/budgets': [budget],
    ...overrides
  }
}

async function mockMonthlyApi(page: Page, overrides: Partial<Record<string, unknown>> = {}) {
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

test('収支を登録する', async ({ page }) => {
  let created: Record<string, unknown> | undefined
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    if (path === '/api/expenses' && request.method() === 'POST') {
      created = { id: 'expense-1', created_at: isoNow, updated_at: isoNow, ...(request.postDataJSON() as Record<string, unknown>) }
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(created) })
      return
    }
    const responses = baseResponses()
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })

  await page.goto('/')
  await page.getByRole('button', { name: '収支を登録' }).click()
  await expect(page.getByRole('heading', { name: '収支を登録' })).toBeVisible()
  await page.getByLabel('金額').fill('1500')
  await page.getByLabel('カテゴリ').click()
  await page.getByRole('option', { name: '食費' }).click()
  await page.getByLabel('支払方法', { exact: false }).click()
  await page.getByRole('option', { name: '現金' }).click()
  await page.getByRole('button', { name: '登録する' }).click()

  await expect(page.getByText('支出を登録しました', { exact: true })).toBeVisible()
  expect(created).toMatchObject({ amount: '1500', category_id: 'expense-category-1', payment_method_id: 'payment-method-1' })
})

test('準固定費の今月分を登録する', async ({ page }) => {
  let created: Record<string, unknown> | undefined
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    if (path === '/api/expenses' && request.method() === 'POST') {
      created = { id: 'expense-1', created_at: isoNow, updated_at: isoNow, ...(request.postDataJSON() as Record<string, unknown>) }
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(created) })
      return
    }
    const responses = baseResponses()
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })

  await page.goto('/')
  await expect(page.getByText('準固定費（金額変動）')).toBeVisible()
  await page.getByRole('button', { name: '電気代の今月分を登録' }).click()
  await expect(page.getByRole('heading', { name: '準固定費を登録' })).toBeVisible()
  await expect(page.getByLabel('金額')).toHaveValue('8000')
  await page.getByLabel('金額').fill('7500')
  await page.getByRole('button', { name: '登録する' }).click()

  await expect(page.getByText('支出を登録しました', { exact: true })).toBeVisible()
  expect(created).toMatchObject({ amount: '7500', recurring_expense_id: 'recurring-1' })
})

test('USD建て準固定費は為替プレビューを表示し、換算後の金額を送信する', async ({ page }) => {
  let created: Record<string, unknown> | undefined
  await page.route('**/api/**', async (route) => {
    const request = route.request()
    const path = new URL(request.url()).pathname
    if (path === '/api/recurring-expenses/recurring-usd-1/exchange-rate') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ foreign_amount: '10', currency_code: 'USD', exchange_rate: '150.5', exchange_rate_date: '2026-08-31', converted_amount: '1505' })
      })
      return
    }
    if (path === '/api/expenses' && request.method() === 'POST') {
      created = request.postDataJSON() as Record<string, unknown>
      await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ id: 'expense-usd-1', created_at: isoNow, updated_at: isoNow, ...created }) })
      return
    }
    const responses = baseResponses({ '/api/recurring-expenses': [usdVariableRecurring] })
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })

  await page.goto('/')
  await page.getByRole('button', { name: '動画サービスの今月分を登録' }).click()
  await expect(page.getByLabel('金額')).toHaveValue('1505')
  await expect(page.getByText(/USD 10 × レート 150.5/)).toBeVisible()
  await page.getByRole('button', { name: '登録する' }).click()

  await expect(page.getByText('支出を登録しました', { exact: true })).toBeVisible()
  expect(created).toMatchObject({ amount: '1505', foreign_amount: '10', currency_code: 'USD', exchange_rate: '150.5' })
})

test('最近の明細から支出を編集・削除する', async ({ page }) => {
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
    const responses = baseResponses({ '/api/expenses': expenses, '/api/recurring-expenses': [] })
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' })
    })
  })

  await page.goto('/')
  await expect(page.getByText('電車代')).toBeVisible()
  await page.getByRole('button', { name: '電車代を編集' }).click()
  await page.getByLabel('金額').fill('3333')
  await page.getByRole('button', { name: '更新する' }).click()
  await expect(page.getByText('支出を更新しました', { exact: true })).toBeVisible()
  await expect(page.getByRole('listitem').filter({ hasText: '電車代' })).toContainText('3,333')

  await page.getByRole('button', { name: '電車代を削除' }).click()
  await page.getByRole('button', { name: '削除', exact: true }).click()
  await expect(page.getByText('明細を削除しました', { exact: true })).toBeVisible()
  await expect(page.getByText('この月の明細はまだありません')).toBeVisible()
})

test('画面切り替えで日別・年間集計へ遷移する', async ({ page }) => {
  await mockMonthlyApi(page)
  await page.goto(`/?month=${month}`)

  await page.getByRole('link', { name: '日別集計' }).click()
  await expect(page).toHaveURL(new RegExp(`/daily\\?month=${month}$`))

  await page.getByRole('link', { name: '年間' }).click()
  await expect(page).toHaveURL(new RegExp(`/annual\\?year=${month.slice(0, 4)}$`))
})
