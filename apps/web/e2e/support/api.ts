import type { Page, Request } from "@playwright/test";

/** GET responses keyed by pathname (e.g. "/api/expenses"). */
export type MockResponses = Record<string, unknown>;

export interface MockReply {
  status?: number;
  body?: unknown;
}

/** Custom handlers keyed by "METHOD /pathname" (e.g. "POST /api/expenses"). */
export type MockHandlers = Record<string, (request: Request) => MockReply | Promise<MockReply>>;

/**
 * Routes every `/api/**` request of the page to in-memory responses.
 *
 * `responses` may be a function so that tests can change the served data after a mutation.
 * Paths without a response or handler reply with 404, like the real API.
 */
export async function mockApi(
  page: Page,
  responses: MockResponses | (() => MockResponses),
  handlers: MockHandlers = {},
) {
  await page.route("**/api/**", async (route) => {
    const request = route.request();
    const path = new URL(request.url()).pathname;
    const handler = handlers[`${request.method()} ${path}`];
    if (handler) {
      const reply = await handler(request);
      const status = reply.status ?? 200;
      await route.fulfill(
        reply.body === undefined
          ? { status }
          : { status, contentType: "application/json", body: JSON.stringify(reply.body) },
      );
      return;
    }

    const current = typeof responses === "function" ? responses() : responses;
    const body = current[path];
    await route.fulfill({
      status: body === undefined ? 404 : 200,
      contentType: "application/json",
      body: JSON.stringify(body ?? { message: "Not found" }),
    });
  });
}

/** The JSON body of a request, typed for assertions. */
export function jsonBody(request: Request): Record<string, unknown> {
  return request.postDataJSON() as Record<string, unknown>;
}
