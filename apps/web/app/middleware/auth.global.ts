import { LOGIN_PATH, isLoginPath, safeRedirectPath } from "~/utils/auth";

export default defineNuxtRouteMiddleware(async (to) => {
  const session = await useAuth().ensure();
  const onLoginPage = isLoginPath(to.path);

  if (session.mode === "none") {
    return onLoginPage ? navigateTo("/", { replace: true }) : undefined;
  }
  if (session.authenticated) {
    return onLoginPage
      ? navigateTo(safeRedirectPath(to.query.redirect), { replace: true })
      : undefined;
  }
  if (!onLoginPage) {
    return navigateTo({ path: LOGIN_PATH, query: { redirect: to.fullPath } }, { replace: true });
  }
});
