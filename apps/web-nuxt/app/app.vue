<script setup lang="ts">
import { ja } from '@nuxt/ui/locale'

useHead({
  htmlAttrs: { lang: 'ja' },
  titleTemplate: title => title && title !== 'Kakeibo' ? `${title} | Kakeibo` : 'Kakeibo',
  link: [{ rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' }]
})
useSeoMeta({
  title: 'Kakeibo',
  description: 'Kakeibo — 日々の収支と予算を管理する家計簿。'
})
</script>

<template>
  <UApp :locale="ja">
    <UHeader :toggle="false">
      <template #left>
        <NuxtLink
          to="/"
          class="flex items-center gap-2 rounded-md focus-visible:outline-2"
          aria-label="Kakeibo ホーム"
        >
          <AppLogo class="size-8 shrink-0" />
          <span class="font-heading text-xl font-bold">Kakeibo</span>
        </NuxtLink>
      </template>
      <template #right>
        <UColorModeButton />
        <UButton
          to="/settings/webhook"
          icon="i-lucide-settings"
          aria-label="設定"
          color="neutral"
          variant="ghost"
        />
      </template>
    </UHeader>
    <UMain>
      <NuxtPage />
    </UMain>
  </UApp>
</template>
