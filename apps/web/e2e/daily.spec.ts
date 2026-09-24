import { expect, test, type Page } from "@playwright/test";
import { jsonBody, mockApi, type MockResponses } from "./support/api";
import {
  category,
  expense as expenseFixture,
  incomeCategory,
  listOf,
  paymentMethod,
  recurringExpense,
} from "./support/fixtures";

const expense = expenseFixture();
const fixedRecurring = recurringExpense({
  id: "recurring-fixed-1",
  name: "家賃",
  amount: "80000",
  payment_day: 1,
});

function baseResponses(overrides: MockResponses = {}): MockResponses {
  return {
    "/api/expenses": [expense],
    "/api/incomes": listOf([]),
    "/api/expense-categories": listOf([category({ name: "交通費" })]),
    "/api/income-categories": listOf([incomeCategory()]),
    "/api/payment-methods": listOf([paymentMethod({ name: "VISA" })]),
    "/api/recurring-expenses": [fixedRecurring],
    ...overrides,
  };
}

async function mockDailyApi(page: Page, overrides: MockResponses = {}) {
  await mockApi(page, baseResponses(overrides));
}

test("4つのタブを切り替えられる", async ({ page }) => {
  await mockDailyApi(page);
  await page.goto("/daily");

  await expect(page.getByText("収支サマリー")).toBeVisible();
  await page.getByRole("tab", { name: "支出明細" }).click();
  await expect(page.getByRole("cell", { name: "電車代", exact: true })).toBeVisible();
  await page.getByRole("tab", { name: "収入明細" }).click();
  await expect(page.getByText("この月の収入明細はありません。")).toBeVisible();
  await page.getByRole("tab", { name: "月ごとのカテゴリ別支出" }).click();
  await expect(page.getByText("日ごとのカテゴリ別支出")).toBeVisible();
});

test("支出明細から支出を編集・削除する", async ({ page }) => {
  let expenses = [expense];
  await mockApi(page, () => baseResponses({ "/api/expenses": expenses }), {
    "PUT /api/expenses/expense-1": (request) => {
      expenses = [{ ...expense, ...jsonBody(request) }];
      return { body: expenses[0] };
    },
    "DELETE /api/expenses/expense-1": () => {
      expenses = [];
      return { status: 204 };
    },
  });

  await page.goto("/daily");
  await page.getByRole("tab", { name: "支出明細" }).click();
  await page.getByRole("button", { name: "電車代の操作" }).click();
  await page.getByRole("menuitem", { name: "編集" }).click();
  await expect(page.getByRole("heading", { name: "支出を編集" })).toBeVisible();
  await page.getByLabel("金額").fill("3333");
  await page.getByRole("button", { name: "更新する" }).click();
  await expect(page.getByText("支出を更新しました", { exact: true })).toBeVisible();
  await expect(page.getByRole("cell", { name: /[¥￥]3,333/ })).toBeVisible();

  await page.getByRole("button", { name: "電車代の操作" }).click();
  await page.getByRole("menuitem", { name: "削除" }).click();
  await page.getByRole("button", { name: "削除", exact: true }).click();
  await expect(page.getByText("明細を削除しました", { exact: true })).toBeVisible();
  await expect(page.getByText("この月の支出明細はありません。")).toBeVisible();
});

test("支出明細をキーワードで絞り込む", async ({ page }) => {
  const secondExpense = expenseFixture({ id: "expense-2", description: "書籍代" });
  await mockDailyApi(page, { "/api/expenses": [expense, secondExpense] });
  await page.goto("/daily");
  await page.getByRole("tab", { name: "支出明細" }).click();

  await expect(page.getByRole("cell", { name: "電車代", exact: true })).toBeVisible();
  await expect(page.getByRole("cell", { name: "書籍代", exact: true })).toBeVisible();

  await page.getByLabel("備考を検索").fill("書籍");
  await expect(page.getByRole("cell", { name: "書籍代", exact: true })).toBeVisible();
  await expect(page.getByRole("cell", { name: "電車代", exact: true })).not.toBeVisible();

  await page.getByRole("button", { name: "条件をクリア" }).click();
  await expect(page.getByRole("cell", { name: "電車代", exact: true })).toBeVisible();
});
