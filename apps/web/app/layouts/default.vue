<script setup lang="ts">
import { settingsNavigation } from "~/utils/settings-navigation";
const links = useAppNavigation();

const groups = computed(() => [
  {
    id: "links",
    label: "ページへ移動",
    items: [...links.value.flat(), ...settingsNavigation],
  },
]);
</script>

<template>
  <UDashboardGroup unit="rem">
    <UDashboardSearch :groups="groups" />
    <slot />
    <AppBottomNav />
  </UDashboardGroup>
</template>
