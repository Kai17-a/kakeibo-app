<script setup lang="ts">
import { settingsNavigation } from '~/utils/settings-navigation'

const links = [settingsNavigation]
</script>

<template>
  <UDashboardPanel
    id="settings"
    :ui="{ body: 'py-6 sm:py-8' }"
  >
    <template #header>
      <UDashboardNavbar title="設定">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
      <UDashboardToolbar>
        <UNavigationMenu
          :items="links"
          aria-label="設定タブ"
          highlight
          class="-mx-1 flex-1"
        />
      </UDashboardToolbar>
    </template>
    <template #body>
      <div class="mx-auto flex w-full max-w-4xl min-w-0 flex-col gap-6">
        <NuxtPage />
      </div>
    </template>
  </UDashboardPanel>
</template>
