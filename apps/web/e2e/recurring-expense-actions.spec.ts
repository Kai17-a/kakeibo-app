import { expect, test, type Page } from '@playwright/test';

const now = new Date().toISOString();
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expenseCategory = {
  id: 'expense-category-1',
  name: '住居費',
  description: null,
  created_at: now,
  updated_at: now,
};
const paymentMethod = {
  id: 'payment-method-1',
  name: '口座振替',
  description: null,
  created_at: now,
  updated_at: now,
};
const recurringExpense = {
  id: 'recurring-1',
  created_at: now,
  updated_at: now,
  name: '家賃',
  amount: '98000',
  payment_day: 27,
  start_date: '2026-01-01',
  end_date: null,
  category_id: 'expense-category-1',
  payment_method_id: 'payment-method-1',
  is_active: true,
  is_variable: false,
  description: null,
};

async function mockRecurringApi(page: Page) {
  let recurringExpenses = [recurringExpense];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/recurring-expenses/recurring-1' && request.method() === 'PUT') {
      const input = request.postDataJSON();
      recurringExpenses = [{ ...recurringExpense, ...input }];
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(recurringExpenses[0]),
      });
      return;
    }
    if (path === '/api/recurring-expenses/recurring-1' && request.method() === 'DELETE') {
      recurringExpenses = [];
      await route.fulfill({ status: 204 });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expense-categories': { items: [expenseCategory], pagination },
      '/api/income-categories': { items: [], pagination },
      '/api/payment-methods': { items: [paymentMethod], pagination },
      '/api/recurring-expenses': recurringExpenses,
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

async function openRecurringTab(page: Page) {
  await page.goto('/settings');
  await page.getByRole('tab', { name: '定期支出' }).click();
}

test('設定画面で定期支出を更新する', async ({ page }) => {
  await mockRecurringApi(page);
  await openRecurringTab(page);

  await page.getByRole('button', { name: '定期支出 家賃を編集' }).click();
  await expect(page.getByRole('heading', { name: '固定費を編集' })).toBeVisible();
  await expect(page.getByLabel('金額', { exact: true })).toHaveValue('98000');
  await page.getByLabel('金額', { exact: true }).fill('100000');
  await page.getByRole('button', { name: '固定費を更新' }).click();

  await expect(page.getByText('定期支出を更新しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: /100,000/ })).toBeVisible();
});

test('設定画面で定期支出を削除する', async ({ page }) => {
  await mockRecurringApi(page);
  await openRecurringTab(page);

  await page.getByRole('button', { name: '定期支出 家賃を削除' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: '削除' }).click();

  await expect(page.getByText('定期支出を削除しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: '家賃' })).not.toBeVisible();
});
