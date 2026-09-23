import { expect, test, type Page } from '@playwright/test';

const now = new Date().toISOString();
const month = `${new Date().getFullYear()}-${String(new Date().getMonth() + 1).padStart(2, '0')}`;
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expenseCategory = {
  id: 'expense-category-1',
  name: '食費',
  description: null,
  parent_category_id: null,
  display_order: 0,
  created_at: now,
  updated_at: now,
};
const paymentMethod = {
  id: 'payment-method-1',
  name: 'VISA',
  description: null,
  created_at: now,
  updated_at: now,
};

async function mockLedgerApi(page: Page, onExpense: (input: Record<string, unknown>) => void) {
  let expenseCategories = [expenseCategory];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/expense-categories' && request.method() === 'POST') {
      const input = request.postDataJSON();
      const category = {
        ...expenseCategory,
        id: 'expense-category-new',
        display_order: expenseCategories.length,
        ...input,
      };
      expenseCategories = [...expenseCategories, category];
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify(category),
      });
      return;
    }
    if (path === '/api/expenses' && request.method() === 'POST') {
      const input = request.postDataJSON() as Record<string, unknown>;
      onExpense(input);
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({ id: 'expense-1', created_at: now, updated_at: now, ...input }),
      });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expenses': [],
      '/api/incomes': { items: [], pagination },
      '/api/expense-categories': { items: expenseCategories, pagination },
      '/api/income-categories': { items: [], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': [],
      '/api/recurring-incomes': [],
      '/api/budgets': [],
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

test('未登録のカテゴリを入力して支出を登録できる', async ({ page }) => {
  let expenseInput: Record<string, unknown> | undefined;
  await mockLedgerApi(page, (input) => {
    expenseInput = input;
  });

  await page.goto(`/monthly?month=${month}`);
  await page.getByRole('button', { name: '収支を登録' }).click();
  await page.getByRole('button', { name: '食費' }).click();
  await page.getByPlaceholder('カテゴリを検索…').fill('交際費');
  await page.getByRole('option', { name: '「交際費」を新規登録' }).click();

  await expect(page.getByRole('button', { name: '交際費' })).toBeVisible();
  await page.getByLabel('金額').fill('3000');
  await page.getByRole('button', { name: '登録する' }).click();

  await expect(page.getByText('支出を登録しました。')).toBeVisible();
  expect(expenseInput).toMatchObject({ amount: '3000', category_id: 'expense-category-new' });
});
