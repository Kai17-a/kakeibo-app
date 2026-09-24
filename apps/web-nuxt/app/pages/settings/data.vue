<script setup lang="ts">
useSeoMeta({ title: 'データ管理設定' })
</script>

<template>
  <div class="space-y-6">
    <UCard>
      <template #header>
        <h2 class="text-lg font-semibold">
          データのエクスポート
        </h2>
        <p class="mt-2 text-sm text-muted">
          登録済みの収支データをCSVファイル（UTF-8・BOM付き）としてダウンロードできます。
        </p>
      </template>
      <div class="flex flex-col gap-3 sm:flex-row">
        <UButton
          to="/api/export/expenses"
          external
          download
          color="neutral"
          variant="outline"
          icon="i-lucide-download"
        >
          支出データ（CSV）
        </UButton>
        <UButton
          to="/api/export/incomes"
          external
          download
          color="neutral"
          variant="outline"
          icon="i-lucide-download"
        >
          収入データ（CSV）
        </UButton>
      </div>
    </UCard>
    <UCard>
      <template #header>
        <h2 class="text-lg font-semibold">
          フルバックアップ
        </h2>
        <p class="mt-2 text-sm text-muted">
          カテゴリや設定を含むすべてのデータをSQLiteデータベースとしてダウンロードできます。
        </p>
      </template>
      <UButton
        to="/api/backup"
        external
        download
        color="neutral"
        variant="outline"
        icon="i-lucide-download"
      >
        バックアップをダウンロード
      </UButton>
    </UCard>
    <UCard>
      <template #header>
        <h2 class="text-lg font-semibold">
          データのインポート
        </h2>
        <p class="mt-2 text-sm text-muted">
          CSVファイルを選択すると内容を確認してから登録できます。登録前にデータベースへの反映は行われません。
        </p>
      </template>
      <div class="flex flex-col gap-6">
        <div>
          <h3 class="mb-2 text-sm font-semibold">
            支出データ
          </h3>
          <SettingsCsvImportPanel kind="expense" />
        </div>
        <USeparator />
        <div>
          <h3 class="mb-2 text-sm font-semibold">
            収入データ
          </h3>
          <SettingsCsvImportPanel kind="income" />
        </div>
        <USeparator />
        <div>
          <h3 class="mb-2 text-sm font-semibold">
            固定費データ
          </h3>
          <SettingsCsvImportPanel kind="recurring-expense" />
        </div>
      </div>
    </UCard>
  </div>
</template>
