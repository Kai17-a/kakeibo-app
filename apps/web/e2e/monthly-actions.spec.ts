import { expect, test, type Page } from '@playwright/test';

const month = new Date().toISOString().slice(0, 7);
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const income = {
  id: 'income-1',
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
  transaction_date: `${month}-15`,
  amount: '300000',
  category_id: 'income-category-1',
  description: '給与振込',
};
const incomeCategory = {
  id: 'income-category-1',
  name: '給与',
  description: null,
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
};
const expenseCategory = {
  id: 'expense-category-1',
  name: '光熱費',
  description: null,
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
};
const paymentMethod = {
  id: 'payment-method-1',
  name: '口座振替',
  description: null,
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
};
const variableRecurring = {
  id: 'recurring-1',
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
  name: '電気代',
  amount: '8000',
  payment_day: 15,
  start_date: '2026-01-01',
  end_date: null,
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  is_active: true,
  is_variable: true,
  description: null,
};

async function mockMonthlyApi(page: Page) {
  let incomes = [income];
  let expenses: unknown[] = [];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/expenses' && request.method() === 'POST') {
      const input = request.postDataJSON();
      const created = { id: 'expense-1', created_at: now(), updated_at: now(), ...input };
      expenses = [created];
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify(created),
      });
      return;
    }
    if (path === '/api/incomes/income-1' && request.method() === 'PUT') {
      const input = request.postDataJSON();
      incomes = [{ ...income, ...input, updated_at: `${month}-16T00:00:00Z` }];
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(incomes[0]),
      });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expenses': expenses,
      '/api/incomes': { items: incomes, pagination },
      '/api/expense-categories': { items: [expenseCategory], pagination },
      '/api/income-categories': { items: [incomeCategory], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': [variableRecurring],
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

function now() {
  return new Date().toISOString();
}

test('最近の明細から収入を更新する', async ({ page }) => {
  await mockMonthlyApi(page);
  await page.goto('/');

  await page.getByRole('button', { name: '給与振込を編集' }).click();
  await expect(page.getByRole('heading', { name: '収入を編集' })).toBeVisible();
  await expect(page.getByLabel('金額')).toHaveValue('300000');
  await page.getByLabel('金額').fill('310000');
  await page.getByRole('button', { name: '更新する' }).click();

  await expect(page.getByText('収入を更新しました。')).toBeVisible();
  await expect(page.getByText(/\+.*310,000/)).toBeVisible();
});

test('準固定費の今月分を登録する', async ({ page }) => {
  await mockMonthlyApi(page);
  await page.goto('/');

  await expect(page.getByText('準固定費（金額変動）')).toBeVisible();
  await page.getByRole('button', { name: '電気代の今月分を登録' }).click();
  await expect(page.getByRole('heading', { name: '準固定費を登録' })).toBeVisible();
  await expect(page.getByLabel('日付')).toHaveValue(`${month}-15`);
  await expect(page.getByLabel('金額')).toHaveValue('8000');
  await page.getByLabel('金額').fill('7500');
  await page.getByRole('button', { name: '登録する' }).click();

  await expect(page.getByText('支出を登録しました。')).toBeVisible();
  await expect(page.getByText(/−.*7,500/)).toBeVisible();
});
