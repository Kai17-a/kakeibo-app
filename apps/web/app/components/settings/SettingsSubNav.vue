<script setup lang="ts">
import { findSettingsGroup } from "~/utils/settings-navigation";

const route = useRoute();
const group = computed(() => findSettingsGroup(route.path));
</script>

<template>
  <nav
    v-if="group && group.pages.length > 1"
    :aria-label="`${group.label}の切り替え`"
    class="flex rounded-lg bg-elevated p-1"
  >
    <NuxtLink
      v-for="page in group.pages"
      :key="page.to"
      :to="page.to"
      :aria-current="route.path.replace(/\/$/, '') === page.to ? 'page' : undefined"
      class="flex min-h-[44px] min-w-0 flex-1 items-center justify-center rounded-md px-2 text-center text-sm font-medium focus-visible:outline-2 focus-visible:outline-primary"
      :class="
        route.path.replace(/\/$/, '') === page.to
          ? 'bg-default text-highlighted shadow-sm'
          : 'text-muted hover:text-default'
      "
    >
      {{ page.label }}
    </NuxtLink>
  </nav>
</template>
