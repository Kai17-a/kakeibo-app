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

async function mockMonthlyApi(page: Page) {
  let incomes = [income];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

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
      '/api/expenses': [],
      '/api/incomes': { items: incomes, pagination },
      '/api/expense-categories': { items: [], pagination },
      '/api/income-categories': { items: [incomeCategory], pagination },
      '/api/payment-methods': { items: [], pagination },
      '/api/recurring-expenses': [],
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
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
