import { expect, test } from "@playwright/test";
import { jsonBody, mockApi } from "./support/api";
import { isoNow, listOf, paymentMethod } from "./support/fixtures";

test("定期振替を月末指定で登録・編集・削除できる", async ({ page }) => {
  let items: Record<string, unknown>[] = [];
  const methods = [
    paymentMethod({ id: "bank", name: "銀行" }),
    paymentMethod({ id: "nisa", name: "NISA" }),
  ];
  const requests: Record<string, unknown>[] = [];
  await mockApi(
    page,
    () => ({ "/api/recurring-transfers": items, "/api/payment-methods": listOf(methods) }),
    {
      "POST /api/recurring-transfers": (request) => {
        const body = jsonBody(request);
        requests.push(body);
        const created = { id: "rt1", created_at: isoNow, updated_at: isoNow, ...body };
        items = [created];
        return { status: 201, body: created };
      },
      "PUT /api/recurring-transfers/rt1": (request) => {
        const body = jsonBody(request);
        requests.push(body);
        items = [{ ...items[0], ...body }];
        return { body: items[0] };
      },
      "DELETE /api/recurring-transfers/rt1": () => {
        items = [];
        return { status: 204 };
      },
    },
  );
  await page.goto("/settings/recurring-incomes");
  await page.getByRole("link", { name: "定期振替", exact: true }).click();
  await expect(page).toHaveURL(/recurring-transfers/);
  await page.getByRole("button", { name: "追加", exact: true }).click();
  await page.getByLabel("名称").fill("つみたてNISA");
  await page.getByLabel("金額").fill("30000");
  await page.getByRole("checkbox", { name: "月末（その月の最終日）にする" }).check();
  await page.getByLabel("移動元").click();
  await page.getByRole("option", { name: "銀行" }).click();
  await page.getByLabel("移動先").click();
  await page.getByRole("option", { name: "NISA" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "追加", exact: true }).click();
  await expect.poll(() => requests[0]).toMatchObject({ payment_day: 31 });
  await page.getByRole("button", { name: "つみたてNISAの操作" }).click();
  await page.getByRole("menuitem", { name: "編集" }).click();
  await page.getByLabel("金額").fill("40000");
  await page.getByRole("button", { name: "更新", exact: true }).click();
  await expect.poll(() => requests[1]).toMatchObject({ amount: "40000" });
  await page.getByRole("button", { name: "つみたてNISAの操作" }).click();
  await page.getByRole("menuitem", { name: "削除" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "削除", exact: true }).click();
  await expect(page.getByText("つみたてNISA")).toHaveCount(0);
});

test("支払方法が2件未満の場合は登録できず理由が表示される", async ({ page }) => {
  await mockApi(page, () => ({
    "/api/recurring-transfers": [],
    "/api/payment-methods": listOf([paymentMethod()]),
  }));
  await page.goto("/settings/recurring-transfers");
  await expect(page.getByText("支払方法が2件以上必要です")).toBeVisible();
  await expect(page.getByRole("button", { name: "追加", exact: true })).toBeDisabled();
});
