import { expect, test, type Page } from "@playwright/test";
import { mockApi } from "./support/api";
import { listOf } from "./support/fixtures";

async function mockDataApi(page: Page) {
  const responses = {
    "/api/expense-categories": listOf([]),
    "/api/income-categories": listOf([]),
    "/api/payment-methods": listOf([]),
    "/api/recurring-expenses": [],
    "/api/recurring-incomes": [],
    "/api/budgets": [],
  };
  await mockApi(page, responses, {
    "POST /api/import/expenses/preview": () => ({
      body: {
        rows: [
          {
            transaction_date: "2026-01-15",
            amount: "1200",
            category: "食費",
            category_is_new: true,
            payment_method: "現金",
            payment_method_is_new: true,
            description: "昼食",
          },
        ],
        created_categories: ["食費"],
        created_payment_methods: ["現金"],
      },
    }),
    "POST /api/import/expenses": () => ({
      body: {
        imported: 1,
        created_categories: ["食費"],
        created_payment_methods: ["現金"],
      },
    }),
    "POST /api/import/incomes/preview": () => ({
      body: {
        rows: [
          {
            transaction_date: "2026-01-15",
            amount: "300000",
            category: "給与",
            category_is_new: true,
            description: "1月分",
          },
        ],
        created_categories: ["給与"],
      },
    }),
    "POST /api/import/incomes": () => ({
      body: {
        imported: 1,
        created_categories: ["給与"],
        created_payment_methods: [],
      },
    }),
    "POST /api/import/recurring-expenses/preview": () => ({
      body: {
        rows: [
          {
            name: "家賃",
            amount: "61100",
            currency: "",
            foreign_amount: "",
            payment_day: "1",
            start_date: "2026-01-01",
            end_date: null,
            category: "住居費",
            category_is_new: true,
            payment_method: "口座振替",
            payment_method_is_new: true,
            is_variable: "",
            description: null,
          },
        ],
        created_categories: ["住居費"],
        created_payment_methods: ["口座振替"],
      },
    }),
    "POST /api/import/recurring-expenses": () => ({
      body: {
        imported: 1,
        created_categories: ["住居費"],
        created_payment_methods: ["口座振替"],
      },
    }),
  });
}

test("CSVをプレビューしてから支出データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "支出データ（CSV）を選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "expenses.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,カテゴリ,支払方法,メモ\n2026-01-15,1200,食費,現金,昼食\n"),
  });

  await expect(page.getByText("1件をインポートします")).toBeVisible();
  await expect(page.getByText("新規カテゴリ: 食費")).toBeVisible();
  await expect(page.getByText("新規支払方法: 現金")).toBeVisible();

  await page.getByRole("button", { name: "登録する" }).click();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "1件の支出をインポートしました" }),
  ).toBeVisible();
});

test("CSVをプレビューしてから収入データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "収入データ（CSV）を選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "incomes.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,カテゴリ,メモ\n2026-01-15,300000,給与,1月分\n"),
  });

  await expect(page.getByText("1件をインポートします")).toBeVisible();
  await page.getByRole("button", { name: "登録する" }).click();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "1件の収入をインポートしました" }),
  ).toBeVisible();
});

test("CSVをプレビューしてから固定費データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "固定費データ（CSV）を選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "recurring.csv",
    mimeType: "text/csv",
    buffer: Buffer.from(
      "名称,金額,通貨,外貨金額,支払日,開始日,終了日,カテゴリ,支払方法,金額変動,備考\n家賃,61100,,,1,2026-01-01,,住居費,口座振替,,\n",
    ),
  });

  await expect(page.getByText("1件をインポートします")).toBeVisible();
  await expect(page.getByText("新規カテゴリ: 住居費")).toBeVisible();
  await page.getByRole("button", { name: "キャンセル" }).click();
  await expect(page.getByText("1件をインポートします")).not.toBeVisible();
});
