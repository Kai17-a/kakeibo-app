import { expect, test, type Page } from "@playwright/test";

const isoNow = new Date().toISOString();
const pagination = { page: 1, per_page: 100, total: 1, total_pages: 1 };
const expenseCategory = {
  id: "expense-category-1",
  name: "住居費",
  description: null,
  parent_category_id: null,
  display_order: 0,
  created_at: isoNow,
  updated_at: isoNow,
};
const paymentMethod = {
  id: "payment-method-1",
  name: "口座振替",
  description: null,
  initial_balance: null,
  balance: null,
  created_at: isoNow,
  updated_at: isoNow,
};

async function mockRecurringApi(page: Page) {
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    const responses: Record<string, unknown> = {
      "/api/expense-categories": { items: [expenseCategory], pagination },
      "/api/income-categories": { items: [], pagination },
      "/api/payment-methods": { items: [paymentMethod], pagination },
      "/api/recurring-expenses": [],
    };
    await route.fulfill({
      status: responses[path] === undefined ? 404 : 200,
      contentType: "application/json",
      body: JSON.stringify(responses[path] ?? { message: "Not found" }),
    });
  });
}

// Regression test for https://github.com/Kai17-a/kakeibo-app issue #25:
// the name field's `autofocus` caused a blur-triggered validation message to
// shift the layout mid-click, making the USD checkbox unclickable whenever the
// name field was still empty. See PR that removed `autofocus` for the root cause.
test("名称が未入力の状態でも外貨建てチェックボックスを切り替えられる", async ({ page }) => {
  await mockRecurringApi(page);
  await page.goto("/settings/recurring-expenses");

  await page.getByRole("button", { name: "追加" }).click();
  const checkbox = page.getByRole("checkbox", { name: "外貨建て（USD）にする" });
  await expect(checkbox).toBeVisible();
  await expect(checkbox).toHaveAttribute("aria-checked", "false");

  await checkbox.click();

  await expect(checkbox).toHaveAttribute("aria-checked", "true");
  await expect(page.getByLabel("毎月のUSD金額")).toBeVisible();
});

test("外貨建てチェックボックスをオフに戻せる", async ({ page }) => {
  await mockRecurringApi(page);
  await page.goto("/settings/recurring-expenses");

  await page.getByRole("button", { name: "追加" }).click();
  const checkbox = page.getByRole("checkbox", { name: "外貨建て（USD）にする" });
  await checkbox.click();
  await expect(checkbox).toHaveAttribute("aria-checked", "true");

  await checkbox.click();
  await expect(checkbox).toHaveAttribute("aria-checked", "false");
  await expect(page.getByLabel("金額", { exact: true })).toBeVisible();
});
