export default defineNuxtPlugin(() => {
  let mode: string | null = null;
  try {
    mode = localStorage.getItem("kakeibo-color-mode") || localStorage.getItem("nuxt-color-mode");
  } catch {
    /* Storage may be unavailable. */
  }
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const dark = useState(
    "app-dark",
    () => mode === "dark" || ((!mode || mode === "system") && media.matches),
  );
  function apply() {
    document.documentElement.dataset.bsTheme = dark.value ? "dark" : "light";
    document.documentElement.classList.toggle("dark", dark.value);
  }
  apply();
  const onChange = () => {
    let saved: string | null = null;
    try {
      saved = localStorage.getItem("kakeibo-color-mode");
    } catch {}
    if (!saved && (!mode || mode === "system")) {
      dark.value = media.matches;
      apply();
    }
  };
  media.addEventListener("change", onChange);
});
