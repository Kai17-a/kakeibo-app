import { LOGIN_PATH } from "~/utils/auth";

export interface AuthSession {
  /** `none` when the server has authentication turned off. */
  mode: "none" | "local";
  authenticated: boolean;
  user: { username: string } | null;
}

const OPEN_SESSION: AuthSession = { mode: "none", authenticated: true, user: null };

/**
 * The sign-in state, loaded once from `/api/auth/session`. The server enforces access; this
 * only decides which screen to show, so a failed lookup falls back to showing the app and any
 * protected API call that returns 401 still leads to the login page.
 */
export function useAuth() {
  const session = useState<AuthSession | null>("auth-session", () => null);

  async function refresh(): Promise<AuthSession> {
    try {
      session.value = await $fetch<AuthSession>("/api/auth/session", {
        retry: 0,
        timeout: 15000,
      });
    } catch {
      session.value = OPEN_SESSION;
    }
    return session.value;
  }

  async function ensure(): Promise<AuthSession> {
    return session.value ?? refresh();
  }

  async function login(username: string, password: string): Promise<void> {
    await $fetch("/api/auth/login", {
      method: "POST",
      body: { username, password },
      retry: 0,
      timeout: 15000,
    });
    await refresh();
  }

  /** Records that the server no longer accepts the session (it answered 401). */
  function markSignedOut() {
    session.value = { mode: "local", authenticated: false, user: null };
  }

  async function logout(): Promise<void> {
    try {
      await $fetch("/api/auth/logout", { method: "POST", retry: 0, timeout: 15000 });
    } finally {
      markSignedOut();
      await navigateTo(LOGIN_PATH);
    }
  }

  const canLogout = computed(() => session.value?.mode === "local" && session.value.authenticated);

  return { session, ensure, refresh, login, logout, markSignedOut, canLogout };
}
