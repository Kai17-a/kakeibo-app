<template>
  <UDashboardPanel id="home">
    <template #header>
      <UDashboardNavbar title="ホーム">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="space-y-4">
        <p class="text-muted">
          通知先のWebhook URLを管理できます。
        </p>
        <UButton
          to="/settings/webhook"
          icon="i-lucide-settings"
        >
          Webhook設定を開く
        </UButton>
      </div>
    </template>
  </UDashboardPanel>
</template>
