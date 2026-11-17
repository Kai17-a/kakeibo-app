<script setup lang="ts">
import { findSettingsGroup } from "~/utils/settings-navigation";

const route = useRoute();
const group = computed(() => findSettingsGroup(route.path));
</script>

<template>
  <nav
    v-if="group && group.pages.length > 1"
    :aria-label="`${group.label}の切り替え`"
    class="border-b border-default"
  >
    <div class="flex w-fit max-w-full overflow-x-auto">
      <NuxtLink
        v-for="page in group.pages"
        :key="page.to"
        :to="page.to"
        :aria-current="route.path.replace(/\/$/, '') === page.to ? 'page' : undefined"
        class="relative flex shrink-0 items-center justify-center px-3 py-2 text-center text-sm font-medium focus-visible:outline-2 focus-visible:outline-primary max-sm:min-h-[44px]"
        :class="
          route.path.replace(/\/$/, '') === page.to
            ? 'text-primary after:absolute after:inset-x-0 after:bottom-0 after:h-[2px] after:rounded-full after:bg-primary'
            : 'text-muted hover:text-default'
        "
      >
        {{ page.label }}
      </NuxtLink>
    </div>
  </nav>
</template>
