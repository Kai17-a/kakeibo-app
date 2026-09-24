import assert from "node:assert/strict";
import { test } from "node:test";
import { validateWebhook } from "../app/utils/webhook.ts";

test("accepts HTTP/HTTPS URLs with at least one event", () => {
  for (const url of ["http://localhost:9000/hook", "https://example.com/hook"]) {
    assert.deepEqual(validateWebhook({ url, events: ["expense.created"] }), []);
  }
});

test("rejects empty, malformed and non-HTTP URLs", () => {
  for (const url of [
    "",
    "  ",
    "https://",
    "https:example.com",
    "ftp://example.com",
    "javascript:alert(1)",
  ]) {
    assert.ok(
      validateWebhook({ url, events: ["expense.created"] }).some((error) => error.name === "url"),
    );
  }
});

test("requires events and limits description length", () => {
  const errors = validateWebhook({
    url: "https://example.com",
    events: [],
    description: "x".repeat(501),
  });
  assert.deepEqual(
    errors.map((error) => error.name),
    ["events", "description"],
  );
});

test("normalizes surrounding URL whitespace before validation", () => {
  assert.deepEqual(
    validateWebhook({ url: "  https://example.com/hook  ", events: ["budget.exceeded"] }),
    [],
  );
});
