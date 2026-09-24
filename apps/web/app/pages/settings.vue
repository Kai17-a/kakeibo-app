<script setup lang="ts">
import { settingsNavigation } from '~/utils/settings-navigation'

const links = [settingsNavigation]
</script>

<template>
  <UDashboardPanel
    id="settings"
    :ui="{ body: 'lg:py-12' }"
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
      <div class="flex flex-col gap-4 sm:gap-6 lg:gap-12 w-full max-w-5xl mx-auto">
        <NuxtPage />
      </div>
    </template>
  </UDashboardPanel>
</template>
