<script setup lang="ts">
import type { NavigationMenuItem } from "~/types/ui";
import { settingsNavigation } from "~/utils/settings-navigation";
import { isValidMonth, isValidYear } from "~/utils/format";

const open = useState("sidebar-open", () => false);
const searchOpen = ref(false);
const keyword = ref("");
const searchResults = computed(() =>
  groups.value.flatMap((group) => group.items).filter((item) => item.label.includes(keyword.value)),
);
function shortcut(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    searchOpen.value = !searchOpen.value;
  }
}
onMounted(() => document.addEventListener("keydown", shortcut));
onBeforeUnmount(() => document.removeEventListener("keydown", shortcut));
const route = useRoute();
const sidebar = ref<HTMLElement>();
let sidebarTrigger: HTMLElement | null = null;
watch(open, async (value) => {
  if (!window.matchMedia("(max-width: 1023px)").matches) return;
  if (value) {
    sidebarTrigger = document.activeElement as HTMLElement;
    await nextTick();
    sidebar.value?.querySelector<HTMLElement>("a, button")?.focus();
  } else sidebarTrigger?.focus();
});
function sidebarKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    closeSidebar();
    return;
  }
  if (event.key !== "Tab" || !window.matchMedia("(max-width: 1023px)").matches) return;
  const targets = sidebar.value?.querySelectorAll<HTMLElement>("a[href], button:not(:disabled)");
  const first = targets?.[0];
  const last = targets?.[targets.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first?.focus();
  }
}
const closeSidebar = () => {
  open.value = false;
};

const links = computed(() => {
  const month =
    typeof route.query.month === "string" && isValidMonth(route.query.month)
      ? route.query.month
      : undefined;
  const year =
    typeof route.query.year === "string" && isValidYear(route.query.year)
      ? route.query.year
      : undefined;
  const selectedMonth = month ?? (year ? `${year}-01` : undefined);
  const selectedYear = month?.slice(0, 4) ?? year;

  return [
    [
      {
        label: "年間集計",
        icon: "i-lucide-house",
        to: selectedYear ? { path: "/", query: { year: selectedYear } } : "/",
        exact: true,
        onSelect: closeSidebar,
      },
      {
        label: "月間集計",
        icon: "i-lucide-calendar-range",
        to: selectedMonth ? { path: "/monthly", query: { month: selectedMonth } } : "/monthly",
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
  <div class="app-shell-layout" :class="{ 'sidebar-open': open }">
    <button
      v-if="open"
      type="button"
      class="app-sidebar-backdrop"
      aria-label="サイドバーを閉じる"
      @click="closeSidebar"
    />
    <aside
      id="app-sidebar"
      ref="sidebar"
      class="app-sidebar-local"
      aria-label="メニュー"
      @keydown="sidebarKeydown"
    >
      <NuxtLink
        to="/"
        class="flex items-center gap-2 font-serif text-xl font-bold"
        aria-label="Kakeibo ホーム"
        @click="closeSidebar"
        ><AppLogo class="size-8 shrink-0" />Kakeibo</NuxtLink
      >
      <AppButton color="neutral" variant="outline" class="w-full" @click="searchOpen = true"
        >ページを検索 <kbd>⌘/Ctrl K</kbd></AppButton
      >
      <AppNavigationMenu
        :items="links[0] || []"
        orientation="vertical"
        aria-label="メインナビゲーション"
      />
    </aside>
    <slot />
    <AppModal
      v-model:open="searchOpen"
      title="ページへ移動"
      description="ページ名を検索してください"
    >
      <template #body
        ><input
          v-model="keyword"
          class="form-control"
          type="search"
          autofocus
          aria-label="ページ名"
        />
        <nav class="app-navigation-vertical mt-4">
          <NuxtLink
            v-for="(item, index) in searchResults"
            :key="index"
            :to="item.to || '/'"
            @click="
              searchOpen = false;
              closeSidebar();
            "
            >{{ item.label }}</NuxtLink
          >
          <p v-if="!searchResults.length">該当するページはありません</p>
        </nav></template
      >
    </AppModal>
  </div>
</template>
