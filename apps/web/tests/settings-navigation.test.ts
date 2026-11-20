import assert from "node:assert/strict";
import { test } from "node:test";
import { findSettingsGroup, visibleSettingsNavigation } from "../app/utils/settings-navigation.ts";

const labels = (loginEnabled: boolean) =>
  visibleSettingsNavigation(loginEnabled).map((group) => group.label);

test("login settings are listed only when the server has login enabled", () => {
  assert.equal(labels(true).at(-1), "ログイン");
  assert.equal(labels(false).includes("ログイン"), false);
  assert.equal(labels(false).length, labels(true).length - 1);
});

test("findSettingsGroup finds the passkey page", () => {
  assert.equal(findSettingsGroup("/settings/passkeys/")?.label, "ログイン");
});
