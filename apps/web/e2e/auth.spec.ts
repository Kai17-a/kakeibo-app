import { expect, test, type Page } from "@playwright/test";
import { jsonBody, mockApi, type MockResponses } from "./support/api";
import { category, incomeCategory, listOf, paymentMethod } from "./support/fixtures";

const signedOut = { mode: "local", authenticated: false, user: null };
const signedIn = { mode: "local", authenticated: true, user: { username: "kaito" } };

const data: MockResponses = {
  "/api/expenses": [],
  "/api/incomes": listOf([]),
  "/api/transfers": [],
  "/api/expense-categories": listOf([category()]),
  "/api/income-categories": listOf([incomeCategory()]),
  "/api/payment-methods": listOf([paymentMethod()]),
  "/api/recurring-expenses": [],
  "/api/recurring-incomes": [],
  "/api/recurring-transfers": [],
  "/api/budgets": [],
};

/** Mocks a server with local authentication; `state.signedIn` follows login and logout. */
async function mockLocalAuth(page: Page, options: { password?: string; status?: number } = {}) {
  const state = { signedIn: false, logins: 0, logouts: 0 };
  await mockApi(
    page,
    () => ({ ...data, "/api/auth/session": state.signedIn ? signedIn : signedOut }),
    {
      "POST /api/auth/login": (request) => {
        state.logins += 1;
        if (options.status) return { status: options.status, body: { message: "rejected" } };
        if (jsonBody(request).password !== (options.password ?? "correct horse battery")) {
          return { status: 401, body: { message: "invalid username or password" } };
        }
        state.signedIn = true;
        return { status: 204 };
      },
      "POST /api/auth/logout": () => {
        state.logouts += 1;
        state.signedIn = false;
        return { status: 204 };
      },
    },
  );
  return state;
}

test("未ログインならログイン画面に移り、ログイン後に元の画面へ戻る", async ({ page }) => {
  const state = await mockLocalAuth(page);

  await page.goto("/daily");

  await expect(page).toHaveURL(/\/login\?redirect=(%2F|\/)daily\/?$/);
  await expect(page.getByRole("heading", { name: "Kakeibo にログイン" })).toBeVisible();
  await page.getByLabel("ユーザー名").fill("kaito");
  await page.getByLabel("パスワード").fill("correct horse battery");
  await page.getByRole("button", { name: "ログイン" }).click();

  await expect(page).toHaveURL(/\/daily\/?$/);
  expect(state.logins).toBe(1);
});

test("パスワードが違うとエラーを表示し、パスワード欄を空にする", async ({ page }) => {
  await mockLocalAuth(page);
  await page.goto("/login");

  await page.getByLabel("ユーザー名").fill("kaito");
  await page.getByLabel("パスワード").fill("wrong password");
  await page.getByRole("button", { name: "ログイン" }).click();

  await expect(page.getByRole("alert")).toContainText("ユーザー名またはパスワードが違います。");
  await expect(page.getByLabel("パスワード")).toHaveValue("");
  await expect(page).toHaveURL(/\/login\/?$/);
});

test("ロック中はしばらく待つよう案内する", async ({ page }) => {
  await mockLocalAuth(page, { status: 429 });
  await page.goto("/login");

  await page.getByLabel("ユーザー名").fill("kaito");
  await page.getByLabel("パスワード").fill("correct horse battery");
  await page.getByRole("button", { name: "ログイン" }).click();

  await expect(page.getByRole("alert")).toContainText("15分ほど待ってから");
});

test("外部サイトへの redirect は無視してトップへ移る", async ({ page }) => {
  await mockLocalAuth(page);
  await page.goto("/login?redirect=//evil.example.com");

  await page.getByLabel("ユーザー名").fill("kaito");
  await page.getByLabel("パスワード").fill("correct horse battery");
  await page.getByRole("button", { name: "ログイン" }).click();

  await expect(page).toHaveURL("/");
});

test("APIが401を返したらログイン画面に移る", async ({ page }) => {
  await mockApi(
    page,
    { ...data, "/api/auth/session": signedIn },
    {
      "GET /api/expenses": () => ({ status: 401, body: { message: "authentication required" } }),
    },
  );

  await page.goto("/monthly");

  await expect(page).toHaveURL(/\/login\?redirect=(%2F|\/)monthly\/?$/);
});

test("ログアウトするとログイン画面に戻る", async ({ page }) => {
  const state = await mockLocalAuth(page);
  state.signedIn = true;

  await page.goto("/settings/budget");
  await page.getByRole("button", { name: "ログアウト" }).click();

  await expect(page).toHaveURL(/\/login\/?$/);
  expect(state.logouts).toBe(1);
});

test("スマホでは設定一覧からログアウトできる", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const state = await mockLocalAuth(page);
  state.signedIn = true;

  await page.goto("/settings");
  await page.getByRole("button", { name: /ログアウト.*kaito/ }).click();

  await expect(page).toHaveURL(/\/login\/?$/);
  expect(state.logouts).toBe(1);
});

test("認証が無効ならログイン画面を出さず、ログアウトも表示しない", async ({ page }) => {
  await mockApi(page, {
    ...data,
    "/api/auth/session": { mode: "none", authenticated: true, user: null },
  });

  await page.goto("/login");

  await expect(page).toHaveURL("/");
  await expect(page.getByRole("button", { name: "ログアウト" })).toHaveCount(0);
});

test("別オリジンとして拒否されたら KAKEIBO_PUBLIC_URL の確認を案内する", async ({ page }) => {
  await mockLocalAuth(page, { status: 403 });
  await page.goto("/login");

  await page.getByLabel("ユーザー名").fill("kaito");
  await page.getByLabel("パスワード").fill("correct horse battery");
  await page.getByRole("button", { name: "ログイン" }).click();

  await expect(page.getByRole("alert")).toContainText("KAKEIBO_PUBLIC_URL");
});
