<script setup lang="ts">
import type { NavigationMenuItem } from "@nuxt/ui";
import { settingsNavigation } from "~/utils/settings-navigation";
import { isValidMonth } from "~/utils/format";

const open = ref(false);
const route = useRoute();
const closeSidebar = () => {
  open.value = false;
};

const links = computed(() => {
  const month =
    typeof route.query.month === "string" && isValidMonth(route.query.month)
      ? route.query.month
      : undefined;
  const year =
    typeof route.query.year === "string" && /^\d{4}$/.test(route.query.year)
      ? route.query.year
      : undefined;
  const selectedMonth = month ?? (year ? `${year}-01` : undefined);
  const selectedYear = month?.slice(0, 4) ?? year;

  return [
    [
      {
        label: "ホーム",
        icon: "i-lucide-house",
        to: selectedMonth ? { path: "/", query: { month: selectedMonth } } : "/",
        exact: true,
        onSelect: closeSidebar,
      },
      {
        label: "日別集計",
        icon: "i-lucide-calendar-days",
        to: selectedMonth ? { path: "/daily", query: { month: selectedMonth } } : "/daily",
        exact: true,
        onSelect: closeSidebar,
      },
      {
        label: "年間集計",
        icon: "i-lucide-calendar-range",
        to: selectedYear ? { path: "/annual", query: { year: selectedYear } } : "/annual",
        exact: true,
        onSelect: closeSidebar,
      },
      {
        label: "設定",
        icon: "i-lucide-settings",
        to: "/settings",
        active: route.path.startsWith("/settings"),
        onSelect: closeSidebar,
      },
    ],
  ] satisfies NavigationMenuItem[][];
});

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
          <span v-if="!collapsed" class="font-serif text-xl font-bold">Kakeibo</span>
        </NuxtLink>
      </template>
      <template #default="{ collapsed }">
        <UDashboardSearchButton :collapsed="collapsed" class="bg-transparent ring-default" />
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
