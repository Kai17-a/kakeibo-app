<script setup lang="ts">
const links = useAppNavigation();
</script>

<template>
  <div class="hidden items-center gap-2 sm:flex">
    <NuxtLink
      to="/"
      class="shrink-0 rounded-md focus-visible:outline-2"
      aria-label="Kakeibo ホーム"
    >
      <AppLogo class="size-8" />
    </NuxtLink>
    <UNavigationMenu
      :items="links[0]?.slice(0, 3)"
      orientation="horizontal"
      aria-label="メインナビゲーション"
    />
  </div>
</template>
