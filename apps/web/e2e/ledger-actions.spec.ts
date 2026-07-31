import { expect, test, type Page } from '@playwright/test';

const month = new Date().toISOString().slice(0, 7);
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expense = {
  id: 'expense-1',
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
  transaction_date: `${month}-02`,
  amount: '2000',
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  recurring_expense_id: null,
  description: '電車代',
};
const expenseCategory = {
  id: 'expense-category-1',
  name: '交通費',
  description: null,
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
};
const paymentMethod = {
  id: 'payment-method-1',
  name: 'VISA',
  description: null,
  created_at: `${month}-01T00:00:00Z`,
  updated_at: `${month}-01T00:00:00Z`,
};

async function mockLedgerApi(page: Page) {
  let expenses = [expense];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/expenses/expense-1' && request.method() === 'PUT') {
      const input = request.postDataJSON();
      expenses = [{ ...expense, ...input, updated_at: `${month}-02T00:00:00Z` }];
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(expenses[0]),
      });
      return;
    }
    if (path === '/api/expenses/expense-1' && request.method() === 'DELETE') {
      expenses = [];
      await route.fulfill({ status: 204 });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expenses': expenses,
      '/api/incomes': { items: [], pagination },
      '/api/expense-categories': { items: [expenseCategory], pagination },
      '/api/income-categories': { items: [], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': [],
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

async function openExpenseDetails(page: Page) {
  await page.goto('/');
  await page.getByRole('radio', { name: '日別集計' }).click();
  await page.getByRole('tab', { name: '支出明細' }).click();
}

test('支出明細から支出を更新する', async ({ page }) => {
  await mockLedgerApi(page);
  await openExpenseDetails(page);

  await page.getByRole('button', { name: /交通費を編集/ }).click();
  await expect(page.getByRole('heading', { name: '支出を編集' })).toBeVisible();
  await expect(page.getByLabel('金額')).toHaveValue('2000');
  await page.getByLabel('金額').fill('3333');
  await page.getByRole('button', { name: '更新する' }).click();

  await expect(page.getByText('支出を更新しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: '3,333' })).toBeVisible();
});

test('支出明細から支出を削除する', async ({ page }) => {
  await mockLedgerApi(page);
  await openExpenseDetails(page);

  await page.getByRole('button', { name: /交通費を削除/ }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: '削除' }).click();

  await expect(page.getByText('明細を削除しました。')).toBeVisible();
  await expect(page.getByText('この月の支出明細はありません。')).toBeVisible();
});
