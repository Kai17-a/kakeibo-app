<script setup lang="ts">
import type { TableColumn } from "@nuxt/ui";
import type {
  ImportKind,
  ImportResult,
  ExpensePreviewRow,
  IncomePreviewRow,
  RecurringExpensePreviewRow,
  VariableExpensePreviewRow,
} from "~/types/import";
import { formatRecurringPaymentDay } from "~/utils/recurring-payment-day";

const props = defineProps<{ kind: ImportKind }>();

const importLabels: Record<ImportKind, string> = {
  expense: "支出",
  income: "収入",
  "recurring-expense": "固定費",
  "variable-expense": "準固定費（月別金額）",
};
const label = computed(() => importLabels[props.kind]);
const sampleUrl = computed(() => importSampleUrls[props.kind]);

type Row =
  | ExpensePreviewRow
  | IncomePreviewRow
  | RecurringExpensePreviewRow
  | VariableExpensePreviewRow;
const previewing = ref(false);
const importing = ref(false);
const error = ref("");
const rows = ref<Row[]>([]);
const previewCreatedCategories = ref<string[]>([]);
const previewCreatedPaymentMethods = ref<string[]>([]);
const hasPreview = ref(false);
const pendingCsv = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

const { preview, run } = useImportApi();
const toast = useToast();

function recurringRow(row: Row) {
  return row as RecurringExpensePreviewRow;
}
function paymentMethodOf(row: Row) {
  return row as { payment_method?: string; payment_method_is_new?: boolean };
}
function categoryOf(row: Row) {
  return row as { category: string; category_is_new?: boolean };
}

const columns = computed<TableColumn<Row>[]>(() => {
  if (props.kind === "recurring-expense") {
    return [
      { accessorKey: "name", header: "名称" },
      {
        accessorKey: "amount",
        header: "金額",
        meta: { class: { th: "text-right", td: "text-right tabular-nums whitespace-nowrap" } },
      },
      { accessorKey: "currency", header: "通貨" },
      { accessorKey: "payment_day", header: "支払日" },
      { accessorKey: "start_date", header: "開始日" },
      { accessorKey: "category", header: "カテゴリ" },
      { accessorKey: "payment_method", header: "支払方法" },
      { accessorKey: "is_variable", header: "金額変動" },
      { accessorKey: "description", header: "備考" },
    ];
  }
  if (props.kind === "expense") {
    return [
      { accessorKey: "transaction_date", header: "日付" },
      {
        accessorKey: "amount",
        header: "金額",
        meta: { class: { th: "text-right", td: "text-right tabular-nums whitespace-nowrap" } },
      },
      { accessorKey: "category", header: "カテゴリ" },
      { accessorKey: "payment_method", header: "支払方法" },
      { accessorKey: "description", header: "メモ" },
    ];
  }
  if (props.kind === "variable-expense") {
    return [
      { accessorKey: "year_month", header: "年月" },
      { accessorKey: "name", header: "名称" },
      { accessorKey: "transaction_date", header: "計上日" },
      {
        accessorKey: "amount",
        header: "金額",
        meta: { class: { th: "text-right", td: "text-right tabular-nums whitespace-nowrap" } },
      },
      { accessorKey: "category", header: "カテゴリ" },
      { accessorKey: "payment_method", header: "支払方法" },
      { accessorKey: "description", header: "メモ" },
    ];
  }
  return [
    { accessorKey: "transaction_date", header: "日付" },
    {
      accessorKey: "amount",
      header: "金額",
      meta: { class: { th: "text-right", td: "text-right tabular-nums whitespace-nowrap" } },
    },
    { accessorKey: "category", header: "カテゴリ" },
    { accessorKey: "description", header: "メモ" },
  ];
});

function clearPreviewRows() {
  rows.value = [];
  previewCreatedCategories.value = [];
  previewCreatedPaymentMethods.value = [];
}

function resetPreview() {
  closePreview();
  clearPreviewRows();
}

/** Keeps the rows until the dialog has left, so its content doesn't empty while it closes. */
function closePreview() {
  hasPreview.value = false;
  pendingCsv.value = null;
  error.value = "";
}

function onPreviewOpenChange(open: boolean) {
  if (!open && !importing.value) closePreview();
}

function onPreviewAfterLeave() {
  if (!hasPreview.value) clearPreviewRows();
}

function selectFile() {
  fileInput.value?.click();
}

function requestPreview(csv: string) {
  switch (props.kind) {
    case "expense":
      return preview("expense", csv);
    case "income":
      return preview("income", csv);
    case "recurring-expense":
      return preview("recurring-expense", csv);
    case "variable-expense":
      return preview("variable-expense", csv);
  }
}

async function onFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  previewing.value = true;
  error.value = "";
  resetPreview();
  try {
    const csv = await file.text();
    const result = await requestPreview(csv);
    rows.value = result.rows;
    previewCreatedCategories.value = result.created_categories;
    previewCreatedPaymentMethods.value =
      "created_payment_methods" in result ? result.created_payment_methods : [];
    pendingCsv.value = csv;
    hasPreview.value = true;
  } catch (caught) {
    error.value = apiErrorMessage(caught);
  } finally {
    previewing.value = false;
  }
}

function successMessage(result: ImportResult) {
  const created = [
    ...result.created_categories.map((name) => `カテゴリ「${name}」`),
    ...result.created_payment_methods.map((name) => `支払方法「${name}」`),
  ];
  const suffix = created.length > 0 ? `${created.join("、")}を追加しました。` : "";
  return `${result.imported}件の${label.value}をインポートしました。${suffix}`;
}

async function importPreview() {
  if (!pendingCsv.value) return;
  importing.value = true;
  error.value = "";
  try {
    const result = await run(props.kind, pendingCsv.value);
    toast.add({ title: successMessage(result), color: "success" });
    closePreview();
  } catch (caught) {
    error.value = apiErrorMessage(caught);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="flex min-w-0 flex-col gap-3">
    <div class="flex flex-col items-start gap-2 sm:flex-row sm:flex-wrap sm:items-center">
      <UButton
        icon="i-lucide-upload"
        color="neutral"
        variant="outline"
        :loading="previewing"
        :disabled="previewing || importing"
        @click="selectFile"
      >
        {{ label }}データ（CSV）を{{ previewing ? "確認中…" : "選択" }}
      </UButton>
      <input
        ref="fileInput"
        type="file"
        accept=".csv,text/csv"
        class="hidden"
        :disabled="previewing || importing"
        @change="onFileChange"
      />
      <UButton :to="sampleUrl" external download variant="link" size="sm">
        テンプレートをダウンロード
      </UButton>
    </div>

    <UAlert v-if="error && !hasPreview" color="error" :description="error" />

    <UModal
      :open="hasPreview"
      :title="`${label}データのインポートプレビュー`"
      :description="`${rows.length}件をインポートします`"
      :dismissible="!importing"
      :close="!importing"
      :ui="{
        content: 'max-w-6xl',
        body: 'flex min-h-0 min-w-0 flex-col overflow-hidden',
        footer: 'flex-wrap justify-end gap-2',
      }"
      @update:open="onPreviewOpenChange"
      @after:leave="onPreviewAfterLeave"
    >
      <template #body>
        <div class="flex min-h-0 min-w-0 flex-col gap-4">
          <UAlert
            v-if="error"
            class="shrink-0"
            color="error"
            title="インポートできませんでした"
            :description="error"
          />
          <div
            v-if="previewCreatedCategories.length || previewCreatedPaymentMethods.length"
            class="flex shrink-0 flex-wrap gap-x-4 gap-y-1 text-sm"
          >
            <span v-if="previewCreatedCategories.length">
              新規カテゴリ: {{ previewCreatedCategories.join("、") }}
            </span>
            <span v-if="previewCreatedPaymentMethods.length">
              新規支払方法: {{ previewCreatedPaymentMethods.join("、") }}
            </span>
          </div>
          <div class="max-h-[28rem] min-h-0 min-w-0 overflow-auto rounded-md border border-default">
            <UTable
              class="min-w-max"
              :ui="{ th: 'px-4 py-2', td: 'px-4 py-2' }"
              :data="rows"
              :columns="columns"
            >
              <template #amount-cell="{ row }">
                <span v-if="kind === 'recurring-expense'" class="tabular-nums">
                  {{ recurringRow(row.original).amount
                  }}{{
                    recurringRow(row.original).foreign_amount
                      ? ` / ${recurringRow(row.original).foreign_amount}`
                      : ""
                  }}
                </span>
                <span v-else class="tabular-nums">{{ row.original.amount }}</span>
              </template>
              <template v-if="kind === 'recurring-expense'" #currency-cell="{ row }">
                {{ recurringRow(row.original).currency || "JPY" }}
              </template>
              <template v-if="kind === 'recurring-expense'" #payment_day-cell="{ row }">
                {{ formatRecurringPaymentDay(recurringRow(row.original).payment_day) }}
              </template>
              <template #category-cell="{ row }">
                <span>
                  {{ categoryOf(row.original).category }}
                  <UBadge
                    v-if="categoryOf(row.original).category_is_new"
                    color="secondary"
                    variant="soft"
                  >
                    新規
                  </UBadge>
                </span>
              </template>
              <template v-if="kind !== 'income'" #payment_method-cell="{ row }">
                <span>
                  {{ paymentMethodOf(row.original).payment_method }}
                  <UBadge
                    v-if="paymentMethodOf(row.original).payment_method_is_new"
                    color="secondary"
                    variant="soft"
                  >
                    新規
                  </UBadge>
                </span>
              </template>
              <template v-if="kind === 'recurring-expense'" #is_variable-cell="{ row }">
                {{ recurringRow(row.original).is_variable || "なし" }}
              </template>
              <template #description-cell="{ row }">
                {{ row.original.description || "" }}
              </template>
            </UTable>
          </div>
        </div>
      </template>
      <template #footer>
        <UButton color="neutral" variant="outline" :disabled="importing" @click="closePreview">
          キャンセル
        </UButton>
        <UButton :loading="importing" :disabled="importing" @click="importPreview">
          登録する
        </UButton>
      </template>
    </UModal>
  </div>
</template>
