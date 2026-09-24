<script setup lang="ts">
import { ja } from '@nuxt/ui/locale'

useHead({
  htmlAttrs: { lang: 'ja' },
  titleTemplate: title => title && title !== 'Kakeibo' ? `${title} | Kakeibo` : 'Kakeibo',
  link: [{ rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' }],
  meta: [{ name: 'theme-color', content: '#f5f2eb' }]
})
useSeoMeta({
  title: 'Kakeibo',
  description: 'Kakeibo — 日々の収支と予算を管理する家計簿。'
})
</script>

<template>
  <UApp :locale="ja">
    <NuxtLoadingIndicator />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
