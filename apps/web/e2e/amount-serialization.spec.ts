import { expect, test, type Page } from '@playwright/test';

const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expenseCategory = {
  id: 'expense-category-1',
  name: '食費',
  type: 'variable',
  display_order: 1,
  is_active: true,
};
const incomeCategory = {
  id: 'income-category-1',
  name: '給与',
  display_order: 1,
  is_active: true,
};
const paymentMethod = {
  id: 'payment-method-1',
  name: 'VISA',
  display_order: 1,
  is_active: true,
};

async function mockApi(page: Page, onPost: (path: string, body: unknown) => void) {
  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (request.method() === 'POST') {
      const body: unknown = request.postDataJSON();
      onPost(path, body);
      const input = body as Record<string, unknown>;
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({ id: 'created-id', ...input }),
      });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expenses': [],
      '/api/incomes': { items: [], pagination },
      '/api/expense-categories': { items: [expenseCategory], pagination },
      '/api/income-categories': { items: [incomeCategory], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': [],
    };
    const response = responses[path];
    await route.fulfill({
      status: response === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(response ?? { message: 'Not found' }),
    });
  });
}

test('支出金額を文字列としてAPIへ送信する', async ({ page }) => {
  let requestBody: Record<string, unknown> | undefined;
  await mockApi(page, (path, body) => {
    if (path === '/api/expenses') requestBody = body as Record<string, unknown>;
  });

  await page.goto('/');
  await page.getByRole('button', { name: '＋ 支出' }).click();
  await page.getByLabel('金額').fill('3333');
  await page.getByRole('button', { name: '登録する' }).click();

  await expect(page.getByText('支出を登録しました。')).toBeVisible();
  expect(requestBody?.amount).toBe('3333');
  expect(typeof requestBody?.amount).toBe('string');
});

test('固定費金額を文字列としてAPIへ送信する', async ({ page }) => {
  let requestBody: Record<string, unknown> | undefined;
  await mockApi(page, (path, body) => {
    if (path === '/api/recurring-expenses') requestBody = body as Record<string, unknown>;
  });

  await page.goto('/');
  await page.getByRole('button', { name: '＋ 固定費' }).click();
  await page.getByLabel('名称').fill('家賃');
  await page.getByLabel('金額').fill('61100');
  await page.getByRole('button', { name: '固定費を登録' }).click();

  await expect(page.getByText('固定費を登録しました。')).toBeVisible();
  expect(requestBody?.amount).toBe('61100');
  expect(typeof requestBody?.amount).toBe('string');
});
