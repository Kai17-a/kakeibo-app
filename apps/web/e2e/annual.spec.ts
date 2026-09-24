import { expect, test, type Page } from "@playwright/test";
import { mockApi, type MockResponses } from "./support/api";
import {
  category,
  currentYear as year,
  expense,
  income,
  listOf,
  paymentMethod,
} from "./support/fixtures";

const expenses = [
  expense({ transaction_date: `${year}-01-15`, amount: "5000", description: "食料品" }),
  expense({
    id: "expense-2",
    transaction_date: `${year}-02-10`,
    amount: "3000",
    description: "外食",
  }),
];
const incomes = [income({ transaction_date: `${year}-01-25` })];

async function mockAnnualApi(page: Page, overrides: MockResponses = {}) {
  await mockApi(page, {
    "/api/expenses": expenses,
    "/api/incomes": listOf(incomes),
    "/api/expense-categories": listOf([category()]),
    "/api/payment-methods": listOf([paymentMethod({ initial_balance: "10000", balance: "10000" })]),
    "/api/recurring-expenses": [],
    "/api/recurring-incomes": [],
    ...overrides,
  });
}

test("年間サマリー・内訳・テーブルが表示される", async ({ page }) => {
  await mockAnnualApi(page);
  await page.goto("/");

  await expect(page.getByText("年間収入", { exact: true })).toBeVisible();
  await expect(page.getByText(/250,000/).first()).toBeVisible();
  await expect(page.getByText("年間支出", { exact: true })).toBeVisible();
  await expect(page.getByText(/8,000/).first()).toBeVisible();

  await expect(page.getByRole("heading", { name: "年間支出の内訳" })).toBeVisible();
  await expect(page.getByText("食費").first()).toBeVisible();

  await expect(page.getByRole("heading", { name: "月ごとの収支" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "カテゴリ別年間集計" })).toBeVisible();

  await expect(page.getByRole("heading", { name: "資産残高推移" })).toBeVisible();
  await expect(page.getByText("現金", { exact: true })).toBeVisible();
});

test("年選択を切り替えるとURLに反映される", async ({ page }) => {
  await mockAnnualApi(page);
  await page.goto(`/?year=${year}`);

  await page.getByRole("button", { name: "前年" }).click();
  await expect(page).toHaveURL(new RegExp(`/\\?year=${Number(year) - 1}$`));

  await page.getByRole("button", { name: "翌年" }).click();
  await page.getByRole("button", { name: "翌年" }).click();
  await expect(page).toHaveURL(new RegExp(`/\\?year=${Number(year) + 1}$`));
});

test("データがない年は空状態を表示する", async ({ page }) => {
  await mockAnnualApi(page, { "/api/expenses": [], "/api/incomes": listOf([]) });
  await page.goto("/");

  await expect(page.getByText("この年の支出はまだありません。")).toBeVisible();
  await expect(page.getByText("この年の支出はありません。")).toBeVisible();
});

test("旧URLの/annualは年間集計へリダイレクトする", async ({ page }) => {
  await mockAnnualApi(page);
  await page.goto(`/annual?year=${year}`);

  await expect(page).toHaveURL(new RegExp(`/\\?year=${year}$`));
  await expect(page.getByText("年間収入", { exact: true })).toBeVisible();
});
