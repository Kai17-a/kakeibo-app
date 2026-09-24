<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui'
import { settingsNavigation } from '~/utils/settings-navigation'

const open = ref(false)
const closeSidebar = () => {
  open.value = false
}
const links = [[{
  label: 'ホーム',
  icon: 'i-lucide-house',
  to: '/',
  exact: true,
  onSelect: closeSidebar
}, {
  label: '設定',
  icon: 'i-lucide-settings',
  to: '/settings',
  type: 'trigger',
  defaultOpen: true,
  children: settingsNavigation.map(item => ({ ...item, onSelect: closeSidebar }))
}]] satisfies NavigationMenuItem[][]

const groups = computed(() => [{
  id: 'links',
  label: 'ページへ移動',
  items: links.flat()
}])
</script>

<template>
  <UDashboardGroup unit="rem">
    <UDashboardSidebar
      id="default"
      v-model:open="open"
      collapsible
      resizable
      class="bg-elevated/25"
    >
      <template #header="{ collapsed }">
        <NuxtLink
          to="/"
          class="flex items-center gap-2 rounded-md focus-visible:outline-2"
          aria-label="Kakeibo ホーム"
          @click="closeSidebar"
        >
          <AppLogo class="size-8 shrink-0" />
          <span
            v-if="!collapsed"
            class="font-heading text-xl font-bold"
          >Kakeibo</span>
        </NuxtLink>
      </template>
      <template #default="{ collapsed }">
        <UDashboardSearchButton
          :collapsed="collapsed"
          class="bg-transparent ring-default"
        />
        <UNavigationMenu
          :collapsed="collapsed"
          :items="links[0]"
          orientation="vertical"
          aria-label="メインナビゲーション"
          tooltip
          popover
        />
      </template>
      <template #footer>
        <UColorModeButton />
      </template>
    </UDashboardSidebar>
    <UDashboardSearch :groups="groups" />
    <slot />
  </UDashboardGroup>
</template>
