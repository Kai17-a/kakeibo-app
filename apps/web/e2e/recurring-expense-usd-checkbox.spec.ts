import { expect, test, type Page } from "@playwright/test";
import { jsonBody, mockApi } from "./support/api";
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

test("準固定費は目安金額なしで登録できる", async ({ page }) => {
  let submitted: Record<string, unknown> | undefined;
  await mockRecurringApi(page);
  await page.unroute("**/api/**");
  await mockApi(
    page,
    {
      "/api/expense-categories": listOf([category({ name: "住居費" })]),
      "/api/income-categories": listOf([]),
      "/api/payment-methods": listOf([paymentMethod({ name: "口座振替" })]),
      "/api/recurring-expenses": [],
    },
    {
      "POST /api/recurring-expenses": (request) => {
        submitted = jsonBody(request);
        return {
          status: 201,
          body: {
            id: "recurring-1",
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            ...submitted,
          },
        };
      },
    },
  );
  await page.goto("/settings/recurring-expenses");

  await page.getByRole("button", { name: "追加", exact: true }).click();
  await page.getByLabel("名称").fill("電気代");
  await page.getByRole("checkbox", { name: "金額が月ごとに変動する（準固定費）" }).click();
  await expect(page.getByLabel("金額（目安・任意）")).toHaveValue("");
  await page.getByLabel("カテゴリ").click();
  await page.getByRole("option", { name: "住居費" }).click();
  await page.getByLabel("支払方法").click();
  await page.getByRole("option", { name: "口座振替" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "追加", exact: true }).click();

  await expect.poll(() => submitted?.amount).toBeNull();
  expect(submitted).toMatchObject({ name: "電気代", is_variable: true });
});
