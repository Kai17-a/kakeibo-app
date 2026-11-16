<script setup lang="ts">
import { findSettingsGroup, settingsNavigation } from "~/utils/settings-navigation";

const route = useRoute();
const currentGroup = computed(() => findSettingsGroup(route.path));
</script>

<template>
  <UDashboardPanel
    id="settings"
    class="pb-[calc(3.5rem+env(safe-area-inset-bottom))] sm:pb-0"
    :ui="{ body: 'py-6 sm:py-8' }"
  >
    <template #header>
      <UDashboardNavbar
        :title="currentGroup?.label ?? '設定'"
        :toggle="false"
        :ui="{
          title: 'sm:sr-only',
          root: 'sm:h-auto sm:min-h-(--ui-header-height) sm:flex-wrap sm:py-2',
        }"
      >
        <template #leading>
          <UButton
            v-if="currentGroup"
            to="/settings"
            icon="i-lucide-chevron-left"
            aria-label="設定に戻る"
            color="neutral"
            variant="ghost"
            class="min-h-[44px] min-w-[44px] justify-center sm:hidden"
          />
          <AppHeaderNav />
        </template>
        <template #right>
          <UColorModeButton class="hidden sm:inline-flex" />
          <AppHeaderActions />
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="flex w-full min-w-0 gap-6">
        <nav aria-label="設定グループ" class="hidden w-56 shrink-0 space-y-1 sm:block">
          <UButton
            v-for="group in settingsNavigation"
            :key="group.label"
            :to="group.pages[0]!.to"
            :icon="group.icon"
            :label="group.label"
            :variant="currentGroup === group ? 'soft' : 'ghost'"
            :color="currentGroup === group ? 'primary' : 'neutral'"
            :aria-current="currentGroup === group ? 'true' : undefined"
            class="min-h-[44px] w-full"
          />
        </nav>
        <div class="flex min-w-0 flex-1 flex-col gap-6">
          <SettingsSubNav />
          <NuxtPage />
        </div>
      </div>
    </template>
  </UDashboardPanel>
</template>
