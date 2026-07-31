import { expect, test, type Page } from '@playwright/test';

const now = new Date().toISOString();
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const paymentMethod = {
  id: 'payment-method-1',
  name: 'VISA',
  description: null,
  created_at: now,
  updated_at: now,
};

async function mockPaymentMethodApi(page: Page) {
  let paymentMethods = [paymentMethod];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/payment-methods' && request.method() === 'POST') {
      const input = request.postDataJSON();
      const created = { id: 'payment-method-2', created_at: now, updated_at: now, ...input };
      paymentMethods = [...paymentMethods, created];
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify(created),
      });
      return;
    }
    if (path === '/api/payment-methods/payment-method-1' && request.method() === 'PUT') {
      const input = request.postDataJSON();
      paymentMethods = [{ ...paymentMethod, ...input, updated_at: now }];
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(paymentMethods[0]),
      });
      return;
    }
    if (path === '/api/payment-methods/payment-method-1' && request.method() === 'DELETE') {
      paymentMethods = [];
      await route.fulfill({ status: 204 });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expense-categories': { items: [], pagination },
      '/api/income-categories': { items: [], pagination },
      '/api/payment-methods': { items: paymentMethods, pagination },
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

async function openPaymentMethodTab(page: Page) {
  await page.goto('/settings');
  await page.getByRole('tab', { name: '支払方法' }).click();
}

test('支払方法を登録する', async ({ page }) => {
  await mockPaymentMethodApi(page);
  await openPaymentMethodTab(page);

  await page.getByRole('button', { name: '追加' }).click();
  await page.getByLabel('支払方法名').fill('電子マネー');
  await page.getByRole('button', { name: '支払方法を追加' }).click();

  await expect(page.getByText('支払方法を追加しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: '電子マネー' })).toBeVisible();
});

test('支払方法を更新する', async ({ page }) => {
  await mockPaymentMethodApi(page);
  await openPaymentMethodTab(page);

  await page.getByRole('button', { name: 'VISAを編集' }).click();
  await expect(page.getByRole('heading', { name: '支払方法を編集' })).toBeVisible();
  await expect(page.getByLabel('支払方法名')).toHaveValue('VISA');
  await page.getByLabel('支払方法名').fill('VISAカード');
  await page.getByRole('button', { name: '支払方法を更新' }).click();

  await expect(page.getByText('支払方法を更新しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: 'VISAカード' })).toBeVisible();
});

test('支払方法を削除する', async ({ page }) => {
  await mockPaymentMethodApi(page);
  page.on('dialog', (dialog) => dialog.accept());
  await openPaymentMethodTab(page);

  await page.getByRole('button', { name: 'VISAを削除' }).click();

  await expect(page.getByText('支払方法を削除しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: 'VISA' })).not.toBeVisible();
});
