import { expect, test, type Page } from "@playwright/test";
import { mockApi } from "./support/api";
import { listOf } from "./support/fixtures";

async function mockDataApi(page: Page, options: { importExpenseFails?: boolean } = {}) {
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
    "POST /api/import/expenses": () =>
      options.importExpenseFails
        ? { status: 500, body: { message: "インポートに失敗しました" } }
        : {
            body: {
              imported: 1,
              created_categories: ["食費"],
              created_payment_methods: ["現金"],
            },
          },
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
    "POST /api/import/variable-expenses/preview": () => ({
      body: {
        rows: [
          {
            year_month: "2026-02",
            name: "電気代",
            transaction_date: "2026-02-28",
            amount: "12345",
            category: "水道光熱費",
            payment_method: "口座振替",
            description: "2月分",
          },
        ],
        created_categories: [],
        created_payment_methods: [],
      },
    }),
    "POST /api/import/variable-expenses": () => ({
      body: {
        imported: 1,
        created_categories: [],
        created_payment_methods: [],
      },
    }),
    "POST /api/import/transfers/preview": () => ({
      body: {
        rows: [
          {
            transaction_date: "2026-01-15",
            amount: "30000",
            from_payment_method: "カード",
            to_payment_method: "NISA",
            description: "積立",
          },
        ],
      },
    }),
    "POST /api/import/transfers": () => ({
      body: { imported: 1, created_categories: [], created_payment_methods: [] },
    }),
  });
}

test("データ管理の各行に正しい操作リンクがあり、狭い画面でも横スクロールしない", async ({
  page,
}) => {
  await mockDataApi(page);
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/settings/data");

  const imports = [
    ["支出", "/api/import/expenses/sample"],
    ["収入", "/api/import/incomes/sample"],
    ["振替", "/api/import/transfers/sample"],
    ["固定費", "/api/import/recurring-expenses/sample"],
    ["準固定費（月別金額）", "/api/import/variable-expenses/sample"],
  ] as const;
  for (const [label, href] of imports) {
    await expect(page.getByRole("button", { name: `${label}データのCSVを選択` })).toHaveText(
      "CSVを選択",
    );
    await expect(
      page.getByRole("link", { name: `${label}データのテンプレートをダウンロード` }),
    ).toHaveAttribute("href", href);
  }

  const csvDownloads = page.getByRole("link", { name: "CSVをダウンロード" });
  await expect(csvDownloads.nth(0)).toHaveAttribute("href", "/api/export/expenses");
  await expect(csvDownloads.nth(1)).toHaveAttribute("href", "/api/export/incomes");
  await expect(csvDownloads.nth(2)).toHaveAttribute("href", "/api/export/transfers");
  await expect(page.getByRole("link", { name: "バックアップをダウンロード" })).toHaveAttribute(
    "href",
    "/api/backup",
  );
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(375);
});

test("CSVをプレビューしてから振替データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");
  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "振替データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "transfers.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,移動元,移動先,メモ\n2026-01-15,30000,カード,NISA,積立\n"),
  });
  const dialog = page.getByRole("dialog");
  await expect(dialog.getByRole("columnheader", { name: "移動元" })).toBeVisible();
  await expect(dialog.getByText("カード", { exact: true })).toBeVisible();
  await expect(dialog.getByText("NISA", { exact: true })).toBeVisible();
  await dialog.getByRole("button", { name: "登録する" }).click();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "1件の振替をインポートしました" }),
  ).toBeVisible();
});

test("CSVをプレビューしてから支出データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "支出データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "expenses.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,カテゴリ,支払方法,メモ\n2026-01-15,1200,食費,現金,昼食\n"),
  });

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText("1件をインポートします")).toBeVisible();
  await expect(dialog.getByText("新規カテゴリ: 食費")).toBeVisible();
  await expect(dialog.getByText("新規支払方法: 現金")).toBeVisible();

  await dialog.getByRole("button", { name: "登録する" }).click();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "1件の支出をインポートしました" }),
  ).toBeVisible();
});

test("CSVをプレビューしてから収入データをインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "収入データのCSVを選択" }).click(),
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
    page.getByRole("button", { name: "固定費データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "recurring.csv",
    mimeType: "text/csv",
    buffer: Buffer.from(
      "名称,金額,通貨,外貨金額,支払日,開始日,終了日,カテゴリ,支払方法,金額変動,備考\n家賃,61100,,,1,2026-01-01,,住居費,口座振替,,\n",
    ),
  });

  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("1件をインポートします")).toBeVisible();
  await expect(dialog.getByText("新規カテゴリ: 住居費")).toBeVisible();
  await dialog.getByRole("button", { name: "キャンセル" }).click();
  await expect(dialog).not.toBeVisible();
});

test("CSVをプレビューしてから準固定費の月別金額をインポートする", async ({ page }) => {
  await mockDataApi(page);
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "準固定費（月別金額）データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "variable-expenses.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("年月,名称,金額,メモ\n2026-02,電気代,12345,2月分\n"),
  });

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("columnheader", { name: "年月" })).toBeVisible();
  await expect(dialog.getByRole("columnheader", { name: "計上日" })).toBeVisible();
  await expect(dialog.getByText("2026-02", { exact: true })).toBeVisible();
  await expect(dialog.getByText("電気代", { exact: true })).toBeVisible();
  await expect(dialog.getByText("2026-02-28", { exact: true })).toBeVisible();
  await expect(dialog.getByText("水道光熱費", { exact: true })).toBeVisible();
  await expect(dialog.getByText("口座振替", { exact: true })).toBeVisible();
  await expect(dialog.getByText("2月分", { exact: true })).toBeVisible();

  await dialog.getByRole("button", { name: "登録する" }).click();
  await expect(
    page
      .locator('[data-slot="title"]')
      .filter({ hasText: "1件の準固定費（月別金額）をインポートしました" }),
  ).toBeVisible();
});

test("CSVインポート失敗をプレビューモーダル内に表示する", async ({ page }) => {
  await mockDataApi(page, { importExpenseFails: true });
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "支出データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "expenses.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,カテゴリ,支払方法,メモ\n2026-01-15,1200,食費,現金,昼食\n"),
  });

  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "登録する" }).click();
  await expect(dialog.getByText("インポートに失敗しました")).toBeVisible();
});

test("プレビューをEscや外側クリックで閉じると破棄され、選び直したCSVだけを登録する", async ({
  page,
}) => {
  await mockDataApi(page);
  const imported: string[] = [];
  page.on("request", (request) => {
    if (request.method() === "POST" && new URL(request.url()).pathname === "/api/import/expenses") {
      imported.push(request.postData() ?? "");
    }
  });
  await page.goto("/settings/data");

  const dialog = page.getByRole("dialog");
  async function selectCsv(memo: string) {
    const [fileChooser] = await Promise.all([
      page.waitForEvent("filechooser"),
      page.getByRole("button", { name: "支出データのCSVを選択" }).click(),
    ]);
    await fileChooser.setFiles({
      name: "expenses.csv",
      mimeType: "text/csv",
      buffer: Buffer.from(`日付,金額,カテゴリ,支払方法,メモ\n2026-01-15,1200,食費,現金,${memo}\n`),
    });
    await expect(dialog.getByText("1件をインポートします")).toBeVisible();
  }

  await selectCsv("Escで破棄");
  await page.keyboard.press("Escape");
  await expect(dialog).not.toBeVisible();

  await selectCsv("外側クリックで破棄");
  await page.mouse.click(5, 5);
  await expect(dialog).not.toBeVisible();

  await selectCsv("登録する分");
  await dialog.getByRole("button", { name: "登録する" }).click();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "1件の支出をインポートしました" }),
  ).toBeVisible();
  await expect(dialog).not.toBeVisible();

  expect(imported).toHaveLength(1);
  expect(imported[0]).toContain("登録する分");
});

test("画面が低くてもプレビューの表を最後までスクロールでき、操作ボタンが画面内に残る", async ({
  page,
}) => {
  const rows = Array.from({ length: 60 }, (_, index) => ({
    transaction_date: "2026-01-15",
    amount: "1200",
    category: "食費",
    category_is_new: false,
    payment_method: "現金",
    payment_method_is_new: false,
    description: `明細${index + 1}`,
  }));
  await mockDataApi(page);
  await page.route("**/api/import/expenses/preview", (route) =>
    route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({ rows, created_categories: [], created_payment_methods: [] }),
    }),
  );
  await page.setViewportSize({ width: 375, height: 420 });
  await page.goto("/settings/data");

  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "支出データのCSVを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name: "expenses.csv",
    mimeType: "text/csv",
    buffer: Buffer.from("日付,金額,カテゴリ,支払方法,メモ\n"),
  });

  const dialog = page.getByRole("dialog");
  await expect(dialog.getByText("60件をインポートします")).toBeVisible();
  await expect(dialog.getByRole("button", { name: "登録する" })).toBeInViewport({ ratio: 1 });

  // The table's scroll area must fit above the footer; otherwise its last rows are clipped.
  const tableBox = await dialog.locator("div.overflow-auto").first().boundingBox();
  const footerBox = await dialog.locator('[data-slot="footer"]').boundingBox();
  expect(tableBox!.y + tableBox!.height).toBeLessThanOrEqual(footerBox!.y);

  await dialog.getByText("明細60", { exact: true }).scrollIntoViewIfNeeded();
  await expect(dialog.getByText("明細60", { exact: true })).toBeInViewport({ ratio: 1 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(375);
});
