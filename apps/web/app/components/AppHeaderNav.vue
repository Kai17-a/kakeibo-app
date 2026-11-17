<script setup lang="ts">
const links = useAppNavigation();
</script>

<template>
  <div class="hidden items-center gap-2 sm:flex">
    <NuxtLink
      to="/monthly"
      class="shrink-0 rounded-md focus-visible:outline-2"
      aria-label="Kakeibo ホーム"
    >
      <AppLogo class="size-8" />
    </NuxtLink>
    <UNavigationMenu
      :ui="{
        link: 'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary focus-visible:outline-solid',
      }"
      :items="links[0]?.slice(0, 3)"
      orientation="horizontal"
      aria-label="メインナビゲーション"
    />
  </div>
</template>
