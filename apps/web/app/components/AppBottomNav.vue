<script setup lang="ts">
const links = useAppNavigation();
</script>

<template>
  <nav
    aria-label="メインナビゲーション"
    class="fixed inset-x-0 bottom-0 z-30 grid grid-cols-4 border-t border-default bg-default pb-[env(safe-area-inset-bottom)] sm:hidden"
  >
    <NuxtLink
      v-for="item in links[0]"
      :key="item.label"
      :to="item.to"
      :aria-label="item.label"
      :aria-current="item.active ? 'page' : undefined"
      class="flex min-h-14 flex-col items-center justify-center gap-1 text-xs focus-visible:outline-2 focus-visible:outline-primary"
      :class="item.active ? 'text-primary' : 'text-muted'"
    >
      <UIcon :name="item.icon" class="size-5" />
      <span>{{ item.shortLabel }}</span>
    </NuxtLink>
  </nav>
</template>
