import assert from "node:assert/strict";
import { test } from "node:test";
import {
  isAuthApiPath,
  isLoginPath,
  loginErrorMessage,
  notifyUnauthorized,
  safeRedirectPath,
  setUnauthorizedListener,
} from "../app/utils/auth.ts";

test("safeRedirectPath keeps in-app paths", () => {
  assert.equal(safeRedirectPath("/monthly?month=2026-11"), "/monthly?month=2026-11");
  assert.equal(safeRedirectPath("/settings/budget"), "/settings/budget");
});

test("safeRedirectPath falls back to the top page for anything else", () => {
  for (const value of [
    undefined,
    null,
    "",
    ["/monthly"],
    "https://evil.example.com",
    "//evil.example.com",
    "/\\evil.example.com",
    "monthly",
    "/login",
    "/login?redirect=/daily",
  ]) {
    assert.equal(safeRedirectPath(value), "/", String(value));
  }
});

test("isAuthApiPath matches only the auth endpoints", () => {
  assert.equal(isAuthApiPath("/api/auth/login"), true);
  assert.equal(isAuthApiPath("http://localhost:8000/api/auth/session"), true);
  assert.equal(isAuthApiPath("/api/budgets"), false);
  assert.equal(isAuthApiPath("/api/authors"), false);
});

test("loginErrorMessage explains each failure", () => {
  assert.equal(loginErrorMessage(401), "ユーザー名またはパスワードが違います。");
  assert.match(loginErrorMessage(429), /15分ほど待って/);
  assert.equal(loginErrorMessage(400), "ユーザー名とパスワードを入力してください。");
  assert.match(loginErrorMessage(403), /KAKEIBO_PUBLIC_URL/);
  assert.match(loginErrorMessage(undefined), /通信に失敗しました/);
  assert.match(loginErrorMessage(500), /通信に失敗しました/);
});

test("isLoginPath accepts the trailing slash the static server adds", () => {
  assert.equal(isLoginPath("/login"), true);
  assert.equal(isLoginPath("/login/"), true);
  assert.equal(isLoginPath("/login-help"), false);
  assert.equal(safeRedirectPath("/login/"), "/");
});

test("notifyUnauthorized reports 401 from protected APIs only", () => {
  let calls = 0;
  setUnauthorizedListener(() => {
    calls += 1;
  });
  try {
    notifyUnauthorized({ request: "/api/expenses", response: { status: 401 } });
    notifyUnauthorized({
      request: new Request("http://localhost/api/budgets"),
      response: { status: 401 },
    });
    notifyUnauthorized({ request: "/api/expenses", response: { status: 500 } });
    notifyUnauthorized({ request: "/api/auth/login", response: { status: 401 } });
    assert.equal(calls, 2);
  } finally {
    setUnauthorizedListener(undefined);
  }
});
