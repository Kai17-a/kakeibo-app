import { expect, test, type Page } from "@playwright/test";
import { mockApi } from "./support/api";
import { category, listOf, paymentMethod } from "./support/fixtures";

async function mockRecurringApi(page: Page) {
  await mockApi(page, {
    "/api/expense-categories": listOf([category({ name: "住居費" })]),
    "/api/income-categories": listOf([]),
    "/api/payment-methods": listOf([paymentMethod({ name: "口座振替" })]),
    "/api/recurring-expenses": [],
  });
}

// Regression test for https://github.com/Kai17-a/kakeibo-app issue #25:
// the name field's `autofocus` caused a blur-triggered validation message to
// shift the layout mid-click, making the USD checkbox unclickable whenever the
// name field was still empty. See PR that removed `autofocus` for the root cause.
test("名称が未入力の状態でも外貨建てチェックボックスを切り替えられる", async ({ page }) => {
  await mockRecurringApi(page);
  await page.goto("/settings/recurring-expenses");

  await page.getByRole("button", { name: "追加", exact: true }).click();
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

  await page.getByRole("button", { name: "追加", exact: true }).click();
  const checkbox = page.getByRole("checkbox", { name: "外貨建て（USD）にする" });
  await checkbox.click();
  await expect(checkbox).toHaveAttribute("aria-checked", "true");

  await checkbox.click();
  await expect(checkbox).toHaveAttribute("aria-checked", "false");
  await expect(page.getByLabel("金額", { exact: true })).toBeVisible();
});
