<script setup lang="ts">
const dark = useState("app-dark", () => false);
function toggle() {
  dark.value = !dark.value;
  document.documentElement.dataset.bsTheme = dark.value ? "dark" : "light";
  document.documentElement.classList.toggle("dark", dark.value);
  try {
    localStorage.setItem("kakeibo-color-mode", dark.value ? "dark" : "light");
  } catch {
    /* Storage may be unavailable. */
  }
}
</script>
<template>
  <button
    class="btn btn-ghost-secondary"
    type="button"
    :aria-label="dark ? 'ライトモードに切り替え' : 'ダークモードに切り替え'"
    @click="toggle"
  >
    {{ dark ? "☀" : "☾" }}
  </button>
</template>
