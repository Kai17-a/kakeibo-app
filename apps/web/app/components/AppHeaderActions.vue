<script setup lang="ts">
const links = useAppNavigation();
const settings = computed(() => links.value[0]?.[3]);
const auth = useAuth();
</script>

<template>
  <div class="hidden items-center gap-1.5 sm:flex">
    <UButton
      :to="settings?.to"
      :icon="settings?.icon"
      :color="settings?.active ? 'primary' : 'neutral'"
      variant="ghost"
      aria-label="設定"
      :aria-current="settings?.active ? 'page' : undefined"
    />
    <UTooltip v-if="auth.canLogout.value" text="ログアウト">
      <UButton
        icon="i-lucide-log-out"
        color="neutral"
        variant="ghost"
        aria-label="ログアウト"
        @click="auth.logout()"
      />
    </UTooltip>
  </div>
</template>
