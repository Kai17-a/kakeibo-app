<script setup lang="ts">
import { findSettingsGroup, visibleSettingsNavigation } from "~/utils/settings-navigation";

const route = useRoute();
const auth = useAuth();
const currentGroup = computed(() => findSettingsGroup(route.path));
const groupItems = computed(() =>
  visibleSettingsNavigation(auth.session.value?.mode === "local").map((group) => ({
    label: group.label,
    icon: group.icon,
    to: group.pages[0]!.to,
    active: currentGroup.value === group,
  })),
);
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
        <UNavigationMenu
          :ui="{
            link: 'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary focus-visible:outline-solid',
          }"
          :items="groupItems"
          orientation="vertical"
          aria-label="設定グループ"
          class="hidden w-56 shrink-0 sm:block"
        />
        <div class="min-w-0 flex-1">
          <UCard v-if="currentGroup" :ui="{ body: 'space-y-6' }">
            <SettingsSubNav />
            <NuxtPage />
          </UCard>
          <NuxtPage v-else />
        </div>
      </div>
    </template>
  </UDashboardPanel>
</template>
