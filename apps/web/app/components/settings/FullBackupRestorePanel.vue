<script setup lang="ts">
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const open = computed(() => selectedFile.value !== null);
const restoring = ref(false);
const error = ref("");
/** True when no usable answer came back, so the restore may or may not have been applied. */
const resultUnknown = ref(false);

function selectFile() {
  fileInput.value?.click();
}

function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  error.value = "";
  resultUnknown.value = false;
  selectedFile.value = file;
}

function close() {
  if (restoring.value) return;
  selectedFile.value = null;
  error.value = "";
  resultUnknown.value = false;
}

function onOpenChange(value: boolean) {
  if (!value) close();
}

async function restore() {
  if (!selectedFile.value) return;
  restoring.value = true;
  error.value = "";
  resultUnknown.value = false;
  try {
    // No client timeout: giving up early would hide the outcome of a restore still running.
    await $fetch("/api/backup/restore", {
      method: "POST",
      headers: { "content-type": "application/octet-stream" },
      body: selectedFile.value,
      retry: 0,
    });
    sessionStorage.setItem("kakeibo-restore-success", "true");
    window.location.reload();
  } catch (caught) {
    const status = (caught as { status?: number }).status;
    // The API rejects with 4xx only before it touches the data; anything else is inconclusive.
    if (status !== undefined && status >= 400 && status < 500) {
      const message = apiErrorMessage(caught);
      error.value = message.includes("変更され")
        ? message
        : `${message} 現在のデータは変更されていません。`;
    } else {
      resultUnknown.value = true;
    }
    restoring.value = false;
  }
}

function reload() {
  window.location.reload();
}
</script>

<template>
  <div class="flex min-w-0 flex-col items-start gap-3 sm:shrink-0 sm:items-end">
    <UButton
      icon="i-lucide-database-backup"
      color="neutral"
      variant="outline"
      :disabled="restoring"
      aria-label="フルバックアップのDBファイルを選択"
      @click="selectFile"
    >
      DBファイルを選択
    </UButton>
    <input
      ref="fileInput"
      type="file"
      accept=".db,application/vnd.sqlite3,application/x-sqlite3"
      class="hidden"
      :disabled="restoring"
      @change="onFileChange"
    />

    <UModal
      :open="open"
      title="フルバックアップから復元"
      description="現在の家計簿データをバックアップの内容で置き換えます。"
      :dismissible="!restoring"
      :close="!restoring"
      :ui="{ footer: 'flex-wrap justify-end gap-2' }"
      @update:open="onOpenChange"
    >
      <template #body>
        <div class="space-y-4">
          <UAlert
            color="error"
            variant="soft"
            title="現在のデータはすべて置き換えられます"
            description="CSVインポートのような追加・マージではありません。実行後、この画面から元に戻すことはできません。"
          />
          <dl class="space-y-2 text-sm">
            <div>
              <dt class="font-medium text-default">選択したファイル</dt>
              <dd class="mt-0.5 break-all text-muted">{{ selectedFile?.name }}</dd>
            </div>
          </dl>
          <p class="text-sm text-muted">
            復元を始める直前に、現在のデータの退避コピーをサーバーの .restore-backups
            フォルダへ自動作成します（新しい5世代を保持）。
          </p>
          <UAlert v-if="error" color="error" title="復元できませんでした" :description="error" />
          <UAlert
            v-if="resultUnknown"
            color="warning"
            title="復元の結果を確認できませんでした"
            description="サーバーから応答がありませんでした。復元が完了している可能性があります。もう一度実行する前に、ページを再読み込みしてデータを確認してください。"
            :actions="[
              {
                label: 'ページを再読み込み',
                color: 'neutral',
                variant: 'outline',
                onClick: reload,
              },
            ]"
          />
        </div>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" :disabled="restoring" @click="close">
          キャンセル
        </UButton>
        <UButton color="error" :loading="restoring" :disabled="restoring" @click="restore">
          現在のデータを置き換える
        </UButton>
      </template>
    </UModal>
  </div>
</template>
