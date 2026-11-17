<script setup lang="ts">
import { settingsNavigation } from "~/utils/settings-navigation";

const auth = useAuth();

onMounted(() => {
  if (window.matchMedia("(min-width: 640px)").matches) {
    void navigateTo(settingsNavigation[0]!.pages[0]!.to, { replace: true });
  }
});
</script>

<template>
  <nav
    aria-label="設定一覧"
    class="divide-y divide-default rounded-lg border border-default sm:hidden"
  >
    <NuxtLink
      v-for="group in settingsNavigation"
      :key="group.label"
      :to="group.pages[0]!.to"
      class="flex min-h-14 items-center gap-3 p-4 first:rounded-t-lg last:rounded-b-lg hover:bg-elevated focus-visible:outline-2 focus-visible:outline-primary"
    >
      <UIcon :name="group.icon" class="size-5 shrink-0 text-muted" />
      <span class="min-w-0 flex-1">
        <span class="block font-medium text-highlighted">{{ group.label }}</span>
        <span class="block text-xs text-muted">{{ group.description }}</span>
      </span>
      <UIcon name="i-lucide-chevron-right" class="size-5 shrink-0 text-muted" />
    </NuxtLink>
  </nav>
  <div v-if="auth.canLogout.value" class="mt-6 sm:hidden">
    <UButton
      icon="i-lucide-log-out"
      color="neutral"
      variant="outline"
      size="xl"
      block
      @click="auth.logout()"
    >
      ログアウト
      <span v-if="auth.session.value?.user" class="text-muted">
        （{{ auth.session.value.user.username }}）
      </span>
    </UButton>
  </div>
</template>
