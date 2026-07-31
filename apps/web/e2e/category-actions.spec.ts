import { expect, test, type Page } from '@playwright/test';

const now = new Date().toISOString();
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expenseCategory = {
  id: 'expense-category-1',
  name: '食費',
  description: '日常の食費',
  created_at: now,
  updated_at: now,
};
const incomeCategory = {
  id: 'income-category-1',
  name: '給与',
  description: null,
  created_at: now,
  updated_at: now,
};

async function mockCategoryApi(page: Page) {
  let expenseCategories = [expenseCategory];
  let incomeCategories = [incomeCategory];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/expense-categories/expense-category-1' && request.method() === 'DELETE') {
      expenseCategories = [];
      await route.fulfill({ status: 204 });
      return;
    }
    if (path === '/api/income-categories/income-category-1' && request.method() === 'DELETE') {
      incomeCategories = [];
      await route.fulfill({ status: 204 });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expense-categories': { items: expenseCategories, pagination },
      '/api/income-categories': { items: incomeCategories, pagination },
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

test('支出カテゴリを削除する', async ({ page }) => {
  await mockCategoryApi(page);
  page.on('dialog', (dialog) => dialog.accept());
  await page.goto('/settings');

  await page.getByRole('button', { name: '食費を削除' }).click();

  await expect(page.getByText('カテゴリを削除しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: '食費' })).not.toBeVisible();
});

test('収入カテゴリを削除する', async ({ page }) => {
  await mockCategoryApi(page);
  page.on('dialog', (dialog) => dialog.accept());
  await page.goto('/settings');

  await page.getByRole('tab', { name: '収入カテゴリ' }).click();
  await page.getByRole('button', { name: '給与を削除' }).click();

  await expect(page.getByText('カテゴリを削除しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: '給与' })).not.toBeVisible();
});
