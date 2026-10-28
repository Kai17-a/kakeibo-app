<script setup lang="ts">
import { settingsNavigation } from "~/utils/settings-navigation";

const links = [settingsNavigation];
</script>

<template>
  <AppDashboardPanel id="settings" :ui="{ body: 'py-6 sm:py-8' }">
    <template #header>
      <AppDashboardNavbar title="設定">
        <template #leading>
          <AppDashboardSidebarCollapse />
        </template>
        <template #right>
          <AppColorModeButton />
        </template>
      </AppDashboardNavbar>
      <AppDashboardToolbar>
        <AppNavigationMenu :items="links" aria-label="設定タブ" highlight class="-mx-1 flex-1" />
      </AppDashboardToolbar>
    </template>
    <template #body>
      <div class="mx-auto flex w-full max-w-4xl min-w-0 flex-col gap-6">
        <NuxtPage />
      </div>
    </template>
  </AppDashboardPanel>
</template>
