import { expect, test, type Page } from '@playwright/test';

const now = new Date().toISOString();
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const webhookUrl = {
  id: 'webhook-1',
  created_at: now,
  updated_at: now,
  url: 'https://example.com/hook',
  description: '通知先1',
  is_active: true,
};

async function mockWebhookApi(page: Page) {
  let webhookUrls = [webhookUrl];

  await page.route('**/api/**', async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;

    if (path === '/api/webhook-urls' && request.method() === 'POST') {
      const input = request.postDataJSON();
      const created = { id: 'webhook-2', created_at: now, updated_at: now, ...input };
      webhookUrls = [...webhookUrls, created];
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify(created),
      });
      return;
    }
    if (path === '/api/webhook-urls/webhook-1' && request.method() === 'DELETE') {
      webhookUrls = [];
      await route.fulfill({ status: 204 });
      return;
    }

    const responses: Record<string, unknown> = {
      '/api/expense-categories': { items: [], pagination },
      '/api/income-categories': { items: [], pagination },
      '/api/payment-methods': { items: [], pagination },
      '/api/recurring-expenses': [],
      '/api/webhook-urls': webhookUrls,
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: 'application/json',
      body: JSON.stringify(responses[path] ?? { message: 'Not found' }),
    });
  });
}

async function openWebhookTab(page: Page) {
  await page.goto('/settings');
  await page.getByRole('tab', { name: 'Webhook' }).click();
}

test('設定画面でWebhook URLを登録する', async ({ page }) => {
  await mockWebhookApi(page);
  await openWebhookTab(page);

  await page.getByRole('button', { name: '追加' }).click();
  await page.getByLabel('Webhook URL').fill('https://example.com/another-hook');
  await page.getByRole('button', { name: 'Webhook URLを追加' }).click();

  await expect(page.getByText('Webhook URLを登録しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: 'https://example.com/another-hook' })).toBeVisible();
});

test('設定画面でWebhook URLを削除する', async ({ page }) => {
  await mockWebhookApi(page);
  await openWebhookTab(page);

  await page.getByRole('button', { name: 'https://example.com/hookを削除' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: '削除' }).click();

  await expect(page.getByText('Webhook URLを削除しました。')).toBeVisible();
  await expect(page.getByRole('cell', { name: 'https://example.com/hook' })).not.toBeVisible();
});
