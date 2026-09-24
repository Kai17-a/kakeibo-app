import { expect, test, type Page } from "@playwright/test";
import { jsonBody, mockApi, type MockHandlers, type MockResponses } from "./support/api";
import {
  budget,
  category,
  currentMonth as month,
  expense,
  incomeCategory,
  isoNow,
  listOf,
  paymentMethod,
  recurringExpense,
} from "./support/fixtures";

const now = new Date();
const variableRecurring = recurringExpense({ is_variable: true });
const usdVariableRecurring = recurringExpense({
  id: "recurring-usd-1",
  name: "動画サービス",
  amount: "10",
  is_variable: true,
  foreign_amount: "10",
  currency_code: "USD",
});

function baseResponses(overrides: MockResponses = {}): MockResponses {
  return {
    "/api/expenses": [],
    "/api/incomes": listOf([]),
    "/api/expense-categories": listOf([category()]),
    "/api/income-categories": listOf([incomeCategory()]),
    "/api/payment-methods": listOf([paymentMethod()]),
    "/api/recurring-expenses": [variableRecurring],
    "/api/recurring-incomes": [],
    "/api/budgets": [budget()],
    ...overrides,
  };
}

async function mockMonthlyApi(
  page: Page,
  overrides: MockResponses = {},
  handlers: MockHandlers = {},
) {
  await mockApi(page, baseResponses(overrides), handlers);
}

/** Records the posted expense and echoes it back like the API does. */
function captureExpenseCreation() {
  const captured: { body?: Record<string, unknown> } = {};
  const handlers: MockHandlers = {
    "POST /api/expenses": (request) => {
      captured.body = jsonBody(request);
      return {
        body: { id: "expense-1", created_at: isoNow, updated_at: isoNow, ...captured.body },
      };
    },
  };
  return { captured, handlers };
}

test("収支を登録する", async ({ page }) => {
  const { captured, handlers } = captureExpenseCreation();
  await mockMonthlyApi(page, {}, handlers);

  await page.goto("/monthly");
  await page.getByRole("button", { name: "記録する" }).first().click();
  await expect(page.getByRole("heading", { name: "収支を登録" })).toBeVisible();
  await page.getByLabel("金額").fill("1500");
  await page.getByLabel("カテゴリ").click();
  await page.getByRole("option", { name: "食費" }).click();
  await page.getByLabel("支払方法", { exact: false }).click();
  await page.getByRole("option", { name: "現金" }).click();
  await page.getByRole("button", { name: "登録する" }).click();

  await expect(page.getByText("支出を登録しました", { exact: true })).toBeVisible();
  expect(captured.body).toMatchObject({
    amount: "1500",
    category_id: "expense-category-1",
    payment_method_id: "payment-method-1",
  });
});

test("準固定費の今月分を登録する", async ({ page }) => {
  const { captured, handlers } = captureExpenseCreation();
  await mockMonthlyApi(page, {}, handlers);

  await page.goto("/monthly");
  await expect(page.getByText("準固定費（金額変動）")).toBeVisible();
  await page.getByRole("button", { name: "電気代の今月分を登録" }).click();
  await expect(page.getByRole("heading", { name: "準固定費を登録" })).toBeVisible();
  await expect(page.getByLabel("金額")).toHaveValue("8000");
  await page.getByLabel("金額").fill("7500");
  await page.getByRole("button", { name: "登録する" }).click();

  await expect(page.getByText("支出を登録しました", { exact: true })).toBeVisible();
  expect(captured.body).toMatchObject({ amount: "7500", recurring_expense_id: "recurring-1" });
});

test("USD建て準固定費は為替プレビューを表示し、換算後の金額を送信する", async ({ page }) => {
  const { captured, handlers } = captureExpenseCreation();
  await mockMonthlyApi(
    page,
    { "/api/recurring-expenses": [usdVariableRecurring] },
    {
      ...handlers,
      "GET /api/recurring-expenses/recurring-usd-1/exchange-rate": () => ({
        body: {
          foreign_amount: "10",
          currency_code: "USD",
          exchange_rate: "150.5",
          exchange_rate_date: "2026-08-31",
          converted_amount: "1505",
        },
      }),
    },
  );

  await page.goto("/monthly");
  await page.getByRole("button", { name: "動画サービスの今月分を登録" }).click();
  await expect(page.getByLabel("金額")).toHaveValue("1505");
  await expect(page.getByText(/USD 10 × レート 150.5/)).toBeVisible();
  await page.getByRole("button", { name: "登録する" }).click();

  await expect(page.getByText("支出を登録しました", { exact: true })).toBeVisible();
  expect(captured.body).toMatchObject({
    amount: "1505",
    foreign_amount: "10",
    currency_code: "USD",
    exchange_rate: "150.5",
  });
});

test("最近の明細から支出を編集・削除する", async ({ page }) => {
  const original = expense();
  let expenses = [original];
  await mockApi(
    page,
    () => baseResponses({ "/api/expenses": expenses, "/api/recurring-expenses": [] }),
    {
      "PUT /api/expenses/expense-1": (request) => {
        expenses = [{ ...original, ...jsonBody(request) }];
        return { body: expenses[0] };
      },
      "DELETE /api/expenses/expense-1": () => {
        expenses = [];
        return { status: 204 };
      },
    },
  );

  await page.goto("/monthly");
  await expect(page.getByText("電車代")).toBeVisible();
  await page.getByRole("button", { name: "電車代の操作" }).click();
  await page.getByRole("menuitem", { name: "編集" }).click();
  await page.getByLabel("金額").fill("3333");
  await page.getByRole("button", { name: "更新する" }).click();
  await expect(page.getByText("支出を更新しました", { exact: true })).toBeVisible();
  await expect(page.getByRole("listitem").filter({ hasText: "電車代" })).toContainText("3,333");

  await page.getByRole("button", { name: "電車代の操作" }).click();
  await page.getByRole("menuitem", { name: "削除" }).click();
  await page.getByRole("button", { name: "削除", exact: true }).click();
  await expect(page.getByText("明細を削除しました", { exact: true })).toBeVisible();
  await expect(page.getByText("この月の明細はまだありません")).toBeVisible();
});

test("カレンダーで対象月を変更する", async ({ page }) => {
  const targetMonthNumber = now.getMonth() === 0 ? 2 : 1;
  const targetMonth = `${now.getFullYear()}-${String(targetMonthNumber).padStart(2, "0")}`;
  await mockMonthlyApi(page);
  await page.goto(`/monthly?month=${month}`);

  await page.getByRole("button", { name: "対象月" }).click();
  await page
    .getByRole("button", { name: `${now.getFullYear()}年${targetMonthNumber}月`, exact: true })
    .click();

  await expect(page).toHaveURL(new RegExp(`/monthly\\?month=${targetMonth}$`));
});

test("画面切り替えで日別・年間集計へ遷移する", async ({ page }) => {
  await mockMonthlyApi(page);
  await page.goto(`/monthly?month=${month}`);
  const navigation = page.getByRole("navigation", { name: "メインナビゲーション" });

  await navigation.getByRole("link", { name: "日別集計" }).click();
  await expect(page).toHaveURL(new RegExp(`/daily\\?month=${month}$`));

  await navigation.getByRole("link", { name: "年間集計" }).click();
  await expect(page).toHaveURL(new RegExp(`/\\?year=${month.slice(0, 4)}$`));

  await navigation.getByRole("link", { name: "月間集計" }).click();
  await expect(page).toHaveURL(new RegExp(`/monthly\\?month=${month.slice(0, 4)}-01$`));
});
