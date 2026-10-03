import { expect, test } from "@playwright/test";
import { jsonBody, mockApi } from "./support/api";
import { isoNow, listOf, paymentMethod } from "./support/fixtures";

test("投資口座を作成し、編集時に解除できる", async ({ page }) => {
  let methods = [paymentMethod({ id: "cash", name: "現金" })];
  const requests: Record<string, unknown>[] = [];
  await mockApi(page, () => ({ "/api/payment-methods": listOf(methods) }), {
    "POST /api/payment-methods": (request) => {
      const body = jsonBody(request);
      requests.push(body);
      const created = {
        id: "nisa",
        created_at: isoNow,
        updated_at: isoNow,
        balance: null,
        ...body,
      };
      methods = [...methods, created as ReturnType<typeof paymentMethod>];
      return { status: 201, body: created };
    },
    "PUT /api/payment-methods/nisa": (request) => {
      const body = jsonBody(request);
      requests.push(body);
      methods = methods.map((item) => (item.id === "nisa" ? { ...item, ...body } : item));
      return { body: methods.find((item) => item.id === "nisa") };
    },
  });

  await page.goto("/settings/payment-methods");
  await page.getByRole("button", { name: "追加", exact: true }).click();
  await page.getByLabel("支払方法名").fill("NISA");
  await page.getByRole("checkbox", { name: "投資口座として扱う" }).check();
  await page.getByRole("dialog").getByRole("button", { name: "追加", exact: true }).click();
  await expect.poll(() => requests[0]).toMatchObject({ name: "NISA", is_investment: true });

  await page.getByRole("button", { name: "NISAの操作" }).click();
  await page.getByRole("menuitem", { name: "編集" }).click();
  const investment = page.getByRole("checkbox", { name: "投資口座として扱う" });
  await expect(investment).toBeChecked();
  await investment.uncheck();
  await page.getByRole("button", { name: "更新", exact: true }).click();
  await expect.poll(() => requests[1]).toMatchObject({ name: "NISA", is_investment: false });
});
