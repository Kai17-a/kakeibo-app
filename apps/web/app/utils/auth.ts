/** Authentication helpers shared by the login page, the route middleware and the fetch hook. */

export const LOGIN_PATH = "/login";

/** Whether `path` is the login page. Static servers may append a trailing slash. */
export function isLoginPath(path: string): boolean {
  return path.replace(/\/$/, "") === LOGIN_PATH;
}

/**
 * Returns `value` when it is a path inside this app, otherwise the top page. Prevents the
 * `redirect` query of the login page from sending users to another site.
 */
export function safeRedirectPath(value: unknown): string {
  if (typeof value !== "string") return "/";
  if (!value.startsWith("/") || value.startsWith("//") || value.startsWith("/\\")) return "/";
  if (isLoginPath(value.split(/[?#]/, 1)[0] ?? "")) return "/";
  return value;
}

/** Whether `url` points at `/api/auth/*`, whose 401 responses must not trigger a redirect. */
export function isAuthApiPath(url: string): boolean {
  const path = url.startsWith("/") ? url : new URL(url).pathname;
  return path.startsWith("/api/auth/");
}

export function loginErrorMessage(status: number | undefined): string {
  switch (status) {
    case 400:
      return "ユーザー名とパスワードを入力してください。";
    case 401:
      return "ユーザー名またはパスワードが違います。";
    case 429:
      return "ログインの失敗が続いたため、一時的にログインできません。15分ほど待ってからお試しください。";
    default:
      return "通信に失敗しました。接続を確認して、もう一度お試しください。";
  }
}

type UnauthorizedListener = () => void;
let unauthorizedListener: UnauthorizedListener | undefined;

/** Registers what happens when a protected API rejects the session (set by a client plugin). */
export function setUnauthorizedListener(listener: UnauthorizedListener | undefined) {
  unauthorizedListener = listener;
}

/**
 * `onResponseError` hook for API clients. A 401 from anything but `/api/auth/*` means the
 * session expired or was revoked, so the registered listener sends the user to the login page.
 */
export function notifyUnauthorized(context: {
  request: RequestInfo | URL;
  response?: { status: number };
}) {
  if (context.response?.status !== 401) return;
  const { request } = context;
  const url =
    typeof request === "string" ? request : request instanceof URL ? request.href : request.url;
  if (isAuthApiPath(url)) return;
  unauthorizedListener?.();
}
