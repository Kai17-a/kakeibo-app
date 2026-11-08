import { expect, test } from "@playwright/test";
import { mockApi } from "./support/api";
import { category, incomeCategory, paymentMethod, listOf, expense } from "./support/fixtures";

const data = {
  "/api/expenses": [expense()],
  "/api/incomes": listOf([]),
  "/api/transfers": [],
  "/api/expense-categories": listOf([category()]),
  "/api/income-categories": listOf([incomeCategory()]),
  "/api/payment-methods": listOf([paymentMethod(), paymentMethod({ id: "second", name: "銀行" })]),
  "/api/recurring-expenses": [],
  "/api/recurring-incomes": [],
};
test("取得失敗はゼロ件と区別し入力を無効化する", async ({ page }) => {
  await mockApi(page, {});
  await page.goto("/");
  await expect(page.getByText("データを読み込めませんでした")).toBeVisible();
  await expect(page.getByText("年間収入", { exact: true })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "記録する", exact: true })).toBeDisabled();
});
for (const action of ["記録する", "振替"]) {
  test(`${action}の未保存入力は閉じる前に確認する`, async ({ page }) => {
    await mockApi(page, data);
    await page.goto("/");
    await page.getByRole("button", { name: action, exact: true }).click();
    await page.getByRole("spinbutton").fill("1234");
    let prompted = false;
    page.on("dialog", async (dialog) => {
      prompted = true;
      await dialog.dismiss();
    });
    await page.getByRole("button", { name: "キャンセル", exact: true }).click();
    await expect(page.getByRole("spinbutton")).toHaveValue("1234");
    expect(prompted).toBe(true);
  });
}
test("月次グラフの数値表をキーボードから開く", async ({ page }) => {
  await mockApi(page, data);
  await page.goto("/");
  const summary = page.getByText("月別の金額を表で見る", { exact: true });
  await summary.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("table", { name: "カテゴリ別月次推移の金額" })).toBeVisible();
});

for (const width of [320, 390, 1280]) {
  test(`操作ボタンと明細配置 ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 844 });
    await mockApi(page, { ...data, "/api/budgets": [] });
    await page.goto("/monthly");
    const record = page.getByRole("button", { name: "記録する", exact: true });
    await expect(record).toBeEnabled();
    const bounds = await record.boundingBox();
    if (width < 640) {
      expect(bounds!.height).toBeGreaterThanOrEqual(44);
      expect(bounds!.y).toBeGreaterThan(650);
      const title = page.getByRole("heading", { name: "月間集計", exact: true });
      expect(await title.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
      true,
    );
    await page.screenshot({ path: `/tmp/kakeibo-ui-${width}.png`, fullPage: true });
  });
}

test("再試行で表示と入力が復旧する", async ({ page }) => {
  let ready = false;
  await mockApi(page, () => (ready ? data : {}));
  await page.goto("/");
  await expect(page.getByText("データを読み込めませんでした")).toBeVisible();
  ready = true;
  await page.getByRole("button", { name: "再試行", exact: true }).click();
  await expect(page.getByText("年間収入", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "記録する", exact: true })).toBeEnabled();
});
test("初期設定が空なら内部値を表示せず登録を案内する", async ({ page }) => {
  await mockApi(page, { ...data, "/api/payment-methods": listOf([]) });
  await page.goto("/");
  await page.getByRole("button", { name: "記録する", exact: true }).click();
  await expect(page.getByText("__NONE__", { exact: true })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "設定で登録してください" })).toBeVisible();
  await expect(page.getByRole("button", { name: "登録する", exact: true })).toBeDisabled();
});
test("Escapeでも未保存の入力を確認し破棄できる", async ({ page }) => {
  await mockApi(page, data);
  await page.goto("/");
  await page.getByRole("button", { name: "記録する", exact: true }).click();
  await page.getByRole("spinbutton").fill("1234");
  page.once("dialog", (dialog) => dialog.accept());
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
});
