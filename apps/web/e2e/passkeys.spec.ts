import { expect, test, type Page } from "@playwright/test";
import { jsonBody, mockApi, type MockHandlers, type MockResponses } from "./support/api";

const signedOut = { mode: "local", authenticated: false, user: null, passkey: true };
const signedIn = { mode: "local", authenticated: true, user: { username: "kaito" }, passkey: true };
const registrationOptions = {
  publicKey: {
    rp: { id: "127.0.0.1", name: "Kakeibo" },
    user: { id: "AQID", name: "kaito", displayName: "kaito" },
    challenge: "BAUG",
    pubKeyCredParams: [{ type: "public-key", alg: -7 }],
    excludeCredentials: [],
    authenticatorSelection: { residentKey: "required", userVerification: "required" },
  },
};
const loginOptions = {
  publicKey: {
    challenge: "BAUG",
    rpId: "127.0.0.1",
    allowCredentials: [],
    userVerification: "required",
  },
};

/** Replaces the browser's WebAuthn calls with fixed credentials, or a cancellation. */
async function stubAuthenticator(page: Page, options: { cancel?: boolean } = {}) {
  await page.addInitScript((cancel) => {
    // The init script runs inside the page, so its helpers cannot live in module scope.
    // oxlint-disable-next-line unicorn/consistent-function-scoping
    const buffer = (...values: number[]) => new Uint8Array(values).buffer;
    const calls: { create: unknown[]; get: unknown[] } = { create: [], get: [] };
    Object.assign(window, { webauthnCalls: calls });
    // oxlint-disable-next-line unicorn/consistent-function-scoping
    const cancelled = () => Promise.reject(new DOMException("cancelled", "NotAllowedError"));
    Object.defineProperty(navigator, "credentials", {
      configurable: true,
      value: {
        create: (request: CredentialCreationOptions) => {
          calls.create.push(request);
          if (cancel) return cancelled();
          return Promise.resolve({
            id: "Bwg",
            rawId: buffer(7, 8),
            type: "public-key",
            response: {
              attestationObject: buffer(1),
              clientDataJSON: buffer(2),
              getTransports: () => ["internal"],
            },
            getClientExtensionResults: () => ({}),
          });
        },
        get: (request: CredentialRequestOptions) => {
          calls.get.push(request);
          if (cancel) return cancelled();
          return Promise.resolve({
            id: "Bwg",
            rawId: buffer(7, 8),
            type: "public-key",
            response: {
              authenticatorData: buffer(1),
              clientDataJSON: buffer(2),
              signature: buffer(3),
              userHandle: buffer(4),
            },
            getClientExtensionResults: () => ({}),
          });
        },
      },
    });
  }, options.cancel ?? false);
}

async function mockServer(page: Page, extra: MockHandlers = {}, data: MockResponses = {}) {
  const state = { signedIn: false, bodies: [] as Record<string, unknown>[] };
  await mockApi(
    page,
    () => ({ "/api/auth/session": state.signedIn ? signedIn : signedOut, ...data }),
    {
      "POST /api/auth/passkey/challenge": () => ({
        body: { challenge_id: "login-challenge", options: loginOptions },
      }),
      "POST /api/auth/passkey/login": (request) => {
        state.bodies.push(jsonBody(request));
        state.signedIn = true;
        return { status: 204 };
      },
      ...extra,
    },
  );
  return state;
}

test("パスキーでログインして元の画面に戻る", async ({ page }) => {
  await stubAuthenticator(page);
  const state = await mockServer(page);

  await page.goto("/settings/budget");
  await page.getByRole("button", { name: "パスキーでログイン" }).click();

  await expect(page).toHaveURL(/\/settings\/budget\/?$/);
  expect(state.bodies).toEqual([
    {
      challenge_id: "login-challenge",
      credential: {
        id: "Bwg",
        rawId: "Bwg",
        type: "public-key",
        response: {
          authenticatorData: "AQ",
          clientDataJSON: "Ag",
          signature: "Aw",
          userHandle: "BA",
        },
        clientExtensionResults: {},
      },
    },
  ]);
});

test("パスキーの操作をキャンセルしたら理由を表示してログイン画面に留まる", async ({ page }) => {
  await stubAuthenticator(page, { cancel: true });
  await mockServer(page);

  await page.goto("/login");
  await page.getByRole("button", { name: "パスキーでログイン" }).click();

  await expect(page.getByRole("alert")).toContainText("キャンセルされたか、時間切れ");
  await expect(page).toHaveURL(/\/login\/?$/);
});

test("サーバーがパスキーに対応していなければボタンを出さない", async ({ page }) => {
  await stubAuthenticator(page);
  await mockApi(page, { "/api/auth/session": { ...signedOut, passkey: false } });

  await page.goto("/login");

  await expect(page.getByRole("button", { name: "ログイン", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "パスキーでログイン" })).toHaveCount(0);
});

test("設定画面でパスキーを追加・削除できる", async ({ page }) => {
  await stubAuthenticator(page);
  const registered: Record<string, unknown>[] = [];
  const deleted: string[] = [];
  let passkeys = [
    { id: "existing", name: "MacBook", created_at: 1_763_000_000, last_used_at: null },
  ];
  const state = await mockServer(page, {
    "POST /api/passkeys/registration": () => ({
      body: { challenge_id: "registration-challenge", options: registrationOptions },
    }),
    "POST /api/passkeys": (request) => {
      const body = jsonBody(request);
      registered.push(body);
      const created = {
        id: "Bwg",
        name: body.name as string,
        created_at: 1_763_100_000,
        last_used_at: null,
      };
      passkeys = [...passkeys, created];
      return { status: 201, body: created };
    },
    "DELETE /api/passkeys/existing": () => {
      deleted.push("existing");
      passkeys = passkeys.filter((item) => item.id !== "existing");
      return { status: 204 };
    },
  });
  state.signedIn = true;
  await page.route("**/api/passkeys", async (route) => {
    if (route.request().method() !== "GET") return route.fallback();
    await route.fulfill({ contentType: "application/json", body: JSON.stringify(passkeys) });
  });

  await page.goto("/settings/passkeys");
  const list = page.getByRole("list", { name: "パスキー一覧" });
  await expect(list.getByRole("listitem")).toHaveCount(1);
  await expect(list).toContainText("MacBook");
  await expect(list).toContainText("未使用");

  await page.getByLabel("名前").fill("iPhone");
  await page.getByRole("button", { name: "パスキーを追加" }).click();

  await expect(list.getByRole("listitem")).toHaveCount(2);
  await expect(list).toContainText("iPhone");
  expect(registered).toEqual([
    {
      challenge_id: "registration-challenge",
      name: "iPhone",
      credential: {
        id: "Bwg",
        rawId: "Bwg",
        type: "public-key",
        response: { attestationObject: "AQ", clientDataJSON: "Ag", transports: ["internal"] },
        clientExtensionResults: {},
      },
    },
  ]);
  const createCall = await page.evaluate(() => {
    const request = (
      window as unknown as { webauthnCalls: { create: CredentialCreationOptions[] } }
    ).webauthnCalls.create[0]!.publicKey!;
    return {
      challenge: Array.from(new Uint8Array(request.challenge as ArrayBuffer)),
      user: Array.from(new Uint8Array(request.user.id as ArrayBuffer)),
      residentKey: request.authenticatorSelection?.residentKey,
    };
  });
  expect(createCall).toEqual({ challenge: [4, 5, 6], user: [1, 2, 3], residentKey: "required" });

  await page.getByRole("button", { name: "MacBookを削除" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "削除" }).click();

  await expect(list.getByRole("listitem")).toHaveCount(1);
  expect(deleted).toEqual(["existing"]);
});

test("認証が無効なら設定一覧にログインの項目を出さない", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await mockApi(page, {
    "/api/auth/session": { mode: "none", authenticated: true, user: null, passkey: false },
  });

  await page.goto("/settings");

  await expect(page.getByRole("navigation", { name: "設定一覧" })).toContainText("データ管理");
  await expect(page.getByRole("navigation", { name: "設定一覧" })).not.toContainText("ログイン");
});

test("パスキーが未登録なら追加ボタンはフォームの1つだけ", async ({ page }) => {
  await stubAuthenticator(page);
  const state = await mockServer(page, {}, { "/api/passkeys": [] });
  state.signedIn = true;

  await page.goto("/settings/passkeys");

  await expect(page.getByText("パスキーがありません")).toBeVisible();
  await expect(page.getByRole("button", { name: "パスキーを追加" })).toHaveCount(1);
});
