<script setup lang="ts">
useSeoMeta({ title: "データ管理設定" });
</script>

<template>
  <section class="space-y-6" aria-labelledby="data-heading">
    <div class="space-y-1">
      <h2 id="data-heading" class="text-xl font-semibold text-highlighted">データ管理</h2>
      <p class="text-sm text-muted">
        家計簿データの書き出し、バックアップ、CSVからの取り込みを行います。
      </p>
    </div>
    <UCard>
      <template #header>
        <h3 class="text-lg font-semibold">データのエクスポート</h3>
        <p class="mt-2 text-sm text-muted">
          登録済みの収支データをCSVファイル（UTF-8・BOM付き）としてダウンロードできます。
        </p>
      </template>
      <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
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
        <h3 class="text-lg font-semibold">フルバックアップ</h3>
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
        <h3 class="text-lg font-semibold">データのインポート</h3>
        <p class="mt-2 text-sm text-muted">
          CSVファイルを選択すると内容を確認してから登録できます。登録前にデータベースへの反映は行われません。
        </p>
      </template>
      <div class="flex flex-col gap-6">
        <div>
          <h4 class="mb-2 text-sm font-semibold">支出データ</h4>
          <SettingsCsvImportPanel kind="expense" />
        </div>
        <USeparator />
        <div>
          <h4 class="mb-2 text-sm font-semibold">収入データ</h4>
          <SettingsCsvImportPanel kind="income" />
        </div>
        <USeparator />
        <div>
          <h4 class="mb-2 text-sm font-semibold">固定費データ</h4>
          <SettingsCsvImportPanel kind="recurring-expense" />
        </div>
      </div>
    </UCard>
  </section>
</template>
