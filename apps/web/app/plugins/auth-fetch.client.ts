import { LOGIN_PATH, isLoginPath, setUnauthorizedListener } from "~/utils/auth";

/**
 * Sends the user to the login page when the API rejects the session, for example after it
 * expired or the password was changed elsewhere. API clients report 401 responses through
 * `notifyUnauthorized`.
 */
export default defineNuxtPlugin((nuxtApp) => {
  setUnauthorizedListener(() => {
    void nuxtApp.runWithContext(async () => {
      useAuth().markSignedOut();
      const route = useRouter().currentRoute.value;
      if (isLoginPath(route.path)) return;
      await navigateTo({ path: LOGIN_PATH, query: { redirect: route.fullPath } });
    });
  });
});
