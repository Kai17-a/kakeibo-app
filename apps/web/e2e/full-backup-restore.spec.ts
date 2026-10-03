import { expect, test, type Page } from "@playwright/test";
import { mockApi } from "./support/api";

async function chooseBackup(page: Page, name = "kakeibo-backup-20261003.db") {
  const [fileChooser] = await Promise.all([
    page.waitForEvent("filechooser"),
    page.getByRole("button", { name: "フルバックアップのDBファイルを選択" }).click(),
  ]);
  await fileChooser.setFiles({
    name,
    mimeType: "application/vnd.sqlite3",
    buffer: Buffer.from("SQLite format 3\0test"),
  });
}

test("ファイル選択後に影響範囲を示し、キャンセルでは送信しない", async ({ page }) => {
  let requests = 0;
  await mockApi(
    page,
    {},
    {
      "POST /api/backup/restore": () => {
        requests += 1;
        return { body: { message: "フルバックアップを復元しました。" } };
      },
    },
  );
  await page.goto("/settings/data");
  await chooseBackup(page);

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText("kakeibo-backup-20261003.db")).toBeVisible();
  await expect(dialog.getByText("現在のデータはすべて置き換えられます")).toBeVisible();
  await expect(dialog.getByText(/追加・マージではありません/)).toBeVisible();
  await expect(dialog.getByText(/退避コピーをサーバーの \.restore-backups/)).toBeVisible();
  await dialog.getByRole("button", { name: "キャンセル" }).click();
  await expect(dialog).not.toBeVisible();
  expect(requests).toBe(0);
});

test("復元中はモーダルを閉じず、成功後に再読み込みして通知する", async ({ page }) => {
  let finishRestore: (() => void) | undefined;
  let requests = 0;
  await mockApi(
    page,
    {},
    {
      "POST /api/backup/restore": async () => {
        requests += 1;
        await new Promise<void>((resolve) => {
          finishRestore = resolve;
        });
        return { body: { message: "フルバックアップを復元しました。" } };
      },
    },
  );
  await page.goto("/settings/data");
  await chooseBackup(page);
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "現在のデータを置き換える" }).click();
  await expect.poll(() => requests).toBe(1);
  await expect(dialog.getByRole("button", { name: "キャンセル" })).toBeDisabled();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeVisible();
  finishRestore?.();
  await expect(
    page.locator('[data-slot="title"]').filter({ hasText: "フルバックアップを復元しました" }),
  ).toBeVisible();
});

test("復元失敗をモーダル内に表示し、現在データが未変更と伝える", async ({ page }) => {
  await mockApi(
    page,
    {},
    {
      "POST /api/backup/restore": () => ({
        status: 400,
        body: { message: "有効なSQLiteバックアップとして読み込めません。" },
      }),
    },
  );
  await page.goto("/settings/data");
  await chooseBackup(page, "not-sqlite.db");
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "現在のデータを置き換える" }).click();
  await expect(dialog.getByText("復元できませんでした")).toBeVisible();
  await expect(dialog.getByText(/有効なSQLiteバックアップとして読み込めません/)).toBeVisible();
  await expect(dialog.getByText(/現在のデータは変更されていません/)).toBeVisible();
});

test("サーバーの応答がない場合は未変更と断定せず、再読み込みを促す", async ({ page }) => {
  let requests = 0;
  await mockApi(page, {});
  await page.route("**/api/backup/restore", async (route) => {
    requests += 1;
    await route.abort("connectionreset");
  });
  await page.goto("/settings/data");
  await chooseBackup(page);
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "現在のデータを置き換える" }).click();

  await expect(dialog.getByText("復元の結果を確認できませんでした")).toBeVisible();
  await expect(dialog.getByText(/復元が完了している可能性があります/)).toBeVisible();
  await expect(dialog.getByText("復元できませんでした")).toHaveCount(0);
  await expect(dialog.getByText(/現在のデータは変更されていません/)).toHaveCount(0);
  expect(requests).toBe(1);

  await Promise.all([
    page.waitForEvent("load"),
    dialog.getByRole("button", { name: "ページを再読み込み" }).click(),
  ]);
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("サーバー内部エラーでも未変更と断定しない", async ({ page }) => {
  await mockApi(
    page,
    {},
    {
      "POST /api/backup/restore": () => ({
        status: 500,
        body: { message: "Database operation failed" },
      }),
    },
  );
  await page.goto("/settings/data");
  await chooseBackup(page);
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "現在のデータを置き換える" }).click();
  await expect(dialog.getByText("復元の結果を確認できませんでした")).toBeVisible();
  await expect(dialog.getByText(/現在のデータは変更されていません/)).toHaveCount(0);
});

test("復元の行でボタンが折り返さず、送信形式が正しい", async ({ page }) => {
  let contentType: string | undefined;
  await mockApi(
    page,
    {},
    {
      "POST /api/backup/restore": (request) => {
        contentType = request.headers()["content-type"];
        return { body: { message: "フルバックアップを復元しました。" } };
      },
    },
  );
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/settings/data");
  const restoreButton = page.getByRole("button", { name: "フルバックアップのDBファイルを選択" });
  const csvButton = page.getByRole("button", { name: "支出データのCSVを選択" });
  const restoreBox = await restoreButton.boundingBox();
  const csvBox = await csvButton.boundingBox();
  expect(restoreBox!.height).toBe(csvBox!.height);

  await chooseBackup(page);
  await page.getByRole("dialog").getByRole("button", { name: "現在のデータを置き換える" }).click();
  await expect.poll(() => contentType).toBe("application/octet-stream");
});
