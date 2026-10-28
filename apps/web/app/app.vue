<script setup lang="ts">
useHead({
  htmlAttrs: { lang: "ja" },
  titleTemplate: (title) => (title && title !== "Kakeibo" ? `${title} | Kakeibo` : "Kakeibo"),
  link: [
    { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
    { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
    { rel: "manifest", href: "/manifest.webmanifest" },
  ],
  meta: [{ name: "theme-color", content: "#f5f2eb" }],
});
useSeoMeta({
  title: "Kakeibo",
  description: "Kakeibo — 日々の収支と予算を管理する家計簿。",
});
</script>

<template>
  <div>
    <NuxtLoadingIndicator />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <AppToasts />
  </div>
</template>
