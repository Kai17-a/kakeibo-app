<script setup lang="ts">
import type { ImportKind } from "~/types/import";

useSeoMeta({ title: "データ管理設定" });

const downloadGroups = [
  {
    id: "csv-export-heading",
    title: "CSVエクスポート",
    items: [
      {
        name: "支出データ",
        description: "登録済みの支出明細（UTF-8・BOM付き）",
        to: "/api/export/expenses",
        action: "CSVをダウンロード",
      },
      {
        name: "収入データ",
        description: "登録済みの収入明細（UTF-8・BOM付き）",
        to: "/api/export/incomes",
        action: "CSVをダウンロード",
      },
      {
        name: "振替データ",
        description: "登録済みの口座間の振替明細（UTF-8・BOM付き）",
        to: "/api/export/transfers",
        action: "CSVをダウンロード",
      },
    ],
  },
  {
    id: "full-backup-heading",
    title: "フルバックアップ",
    items: [
      {
        name: "すべてのデータ",
        description: "カテゴリや設定を含むSQLiteデータベース全体",
        to: "/api/backup",
        action: "バックアップをダウンロード",
      },
    ],
  },
];

/** Each description lists the CSV columns, matching the template served by the API. */
const importGroups: {
  id: string;
  title: string;
  items: { kind: ImportKind; name: string; description: string }[];
}[] = [
  {
    id: "statement-import-heading",
    title: "明細",
    items: [
      { kind: "expense", name: "支出データ", description: "日付・金額・カテゴリ・支払方法・メモ" },
      { kind: "income", name: "収入データ", description: "日付・金額・カテゴリ・メモ" },
      { kind: "transfer", name: "振替データ", description: "日付・金額・移動元・移動先・メモ" },
    ],
  },
  {
    id: "recurring-import-heading",
    title: "定期の収支",
    items: [
      {
        kind: "recurring-expense",
        name: "固定費データ",
        description:
          "名称・金額・通貨・外貨金額・支払日・開始日・終了日・カテゴリ・支払方法・金額変動・備考",
      },
      {
        kind: "variable-expense",
        name: "準固定費データ（月別金額）",
        description: "年月・名称・金額・メモ",
      },
    ],
  },
];

const toast = useToast();
onMounted(() => {
  if (sessionStorage.getItem("kakeibo-restore-success") !== "true") return;
  sessionStorage.removeItem("kakeibo-restore-success");
  toast.add({ title: "フルバックアップを復元しました", color: "success" });
});
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
        <h3 class="text-lg font-semibold">データのダウンロード</h3>
        <p class="mt-2 text-sm text-muted">
          用途に合わせてCSVファイルまたはデータベース全体をダウンロードできます。
        </p>
      </template>

      <div class="space-y-6">
        <section v-for="group in downloadGroups" :key="group.id" :aria-labelledby="group.id">
          <h4 :id="group.id" class="text-sm font-medium text-muted">{{ group.title }}</h4>
          <div class="mt-2 divide-y divide-default border-t border-default">
            <div
              v-for="item in group.items"
              :key="item.to"
              class="flex min-w-0 flex-col gap-3 py-4 last:pb-0 sm:flex-row sm:items-center sm:justify-between"
            >
              <div class="min-w-0">
                <p class="font-medium text-default">{{ item.name }}</p>
                <p class="mt-0.5 text-sm text-muted">{{ item.description }}</p>
              </div>
              <UButton
                :to="item.to"
                external
                download
                color="neutral"
                variant="outline"
                icon="i-lucide-download"
                class="self-start sm:shrink-0 sm:self-auto"
              >
                {{ item.action }}
              </UButton>
            </div>
          </div>
        </section>
      </div>
    </UCard>

    <UCard>
      <template #header>
        <h3 class="text-lg font-semibold">データのインポート</h3>
        <p class="mt-2 text-sm text-muted">
          CSVは明細を追加し、フルバックアップは現在の全データを置き換えます。
        </p>
      </template>

      <div class="space-y-6">
        <section v-for="group in importGroups" :key="group.id" :aria-labelledby="group.id">
          <h4 :id="group.id" class="text-sm font-medium text-muted">{{ group.title }}</h4>
          <div class="mt-2 divide-y divide-default border-t border-default">
            <div
              v-for="item in group.items"
              :key="item.kind"
              class="flex min-w-0 flex-col gap-3 py-4 last:pb-0 sm:flex-row sm:items-start sm:justify-between"
            >
              <div class="min-w-0">
                <p class="font-medium text-default">{{ item.name }}</p>
                <p class="mt-0.5 text-sm break-words text-muted">{{ item.description }}</p>
              </div>
              <SettingsCsvImportPanel :kind="item.kind" />
            </div>
          </div>
        </section>

        <section aria-labelledby="backup-restore-heading">
          <h4 id="backup-restore-heading" class="text-sm font-medium text-muted">
            データベース全体の復元
          </h4>
          <div class="mt-2 divide-y divide-default border-t border-default">
            <div
              class="flex min-w-0 flex-col gap-3 py-4 last:pb-0 sm:flex-row sm:items-start sm:justify-between"
            >
              <div class="min-w-0">
                <p class="font-medium text-default">フルバックアップの復元</p>
                <p class="mt-0.5 text-sm break-words text-muted">
                  現在の全データをバックアップ（SQLite）の内容で置き換えます。追加・マージではありません。
                </p>
              </div>
              <SettingsFullBackupRestorePanel />
            </div>
          </div>
        </section>
      </div>
    </UCard>
  </section>
</template>
