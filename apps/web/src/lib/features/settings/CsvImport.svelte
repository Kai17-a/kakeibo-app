<script lang="ts">
  import UploadIcon from '@lucide/svelte/icons/upload';
  import { toast } from 'svelte-sonner';
  import { api, importSampleUrls } from '$lib/api';
  import { Badge } from '$lib/components/ui/badge';
  import { Button, buttonVariants } from '$lib/components/ui/button';
  import { Spinner } from '$lib/components/ui/spinner';
  import * as Table from '$lib/components/ui/table';
  import type {
    ExpenseImportPreview,
    ExpensePreviewRow,
    ImportResult,
    IncomeImportPreview,
    IncomePreviewRow,
    RecurringExpenseImportPreview,
    RecurringExpensePreviewRow,
  } from '$lib/types';
  import { cn } from '$lib/utils';

  interface Props {
    kind: 'expense' | 'income' | 'recurring-expense';
    onimported?: () => void;
  }
  type Preview = ExpenseImportPreview | IncomeImportPreview | RecurringExpenseImportPreview;
  let { kind, onimported }: Props = $props();
  let previewing = $state(false);
  let importing = $state(false);
  let error = $state('');
  let preview = $state<Preview | null>(null);
  let pendingCsv = $state<string | null>(null);
  const label = $derived(kind === 'expense' ? '支出' : kind === 'income' ? '収入' : '固定費');
  const sampleUrl = $derived(importSampleUrls[kind]);

  async function previewFile(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    previewing = true;
    error = '';
    preview = null;
    pendingCsv = null;
    try {
      const csv = await file.text();
      preview =
        kind === 'expense'
          ? await api.previewImportExpenses(csv)
          : kind === 'income'
            ? await api.previewImportIncomes(csv)
            : await api.previewImportRecurringExpenses(csv);
      pendingCsv = csv;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'CSVをプレビューできませんでした。';
    } finally {
      previewing = false;
    }
  }

  async function importPreview() {
    if (!pendingCsv || !preview) return;
    importing = true;
    error = '';
    try {
      const result =
        kind === 'expense'
          ? await api.importExpenses(pendingCsv)
          : kind === 'income'
            ? await api.importIncomes(pendingCsv)
            : await api.importRecurringExpenses(pendingCsv);
      toast.success(successMessage(result));
      resetPreview();
      onimported?.();
    } catch (caught) {
      error = caught instanceof Error ? caught.message : 'インポートできませんでした。';
    } finally {
      importing = false;
    }
  }

  function resetPreview() {
    preview = null;
    pendingCsv = null;
    error = '';
  }

  function successMessage(result: ImportResult) {
    const created = [
      ...result.created_categories.map((name) => `カテゴリ「${name}」`),
      ...result.created_payment_methods.map((name) => `支払方法「${name}」`),
    ];
    const suffix = created.length > 0 ? `${created.join('、')}を追加しました。` : '';
    return `${result.imported}件の${label}をインポートしました。${suffix}`;
  }

  function paymentMethods(value: Preview) {
    return 'created_payment_methods' in value ? value.created_payment_methods : [];
  }

  function expenseRow(row: ExpensePreviewRow | IncomePreviewRow | RecurringExpensePreviewRow) {
    return row as ExpensePreviewRow;
  }

  function recurringRow(row: ExpensePreviewRow | IncomePreviewRow | RecurringExpensePreviewRow) {
    return row as RecurringExpensePreviewRow;
  }
</script>

<div class="flex min-w-0 flex-col gap-3">
  <div class="flex flex-wrap items-center gap-2">
    <label
      class={cn(
        buttonVariants({ variant: 'outline' }),
        (previewing || importing) && 'pointer-events-none opacity-50',
      )}
    >
      {#if previewing}<Spinner data-icon="inline-start" />{:else}<UploadIcon
          data-icon="inline-start"
        />{/if}
      {label}データ（CSV）を{previewing ? '確認中…' : '選択'}
      <input
        type="file"
        accept=".csv,text/csv"
        class="hidden"
        disabled={previewing || importing}
        onchange={previewFile}
      />
    </label>
    <a class={buttonVariants({ variant: 'link', size: 'sm' })} href={sampleUrl} download>
      テンプレートをダウンロード
    </a>
  </div>

  {#if error}<p class="text-sm whitespace-pre-line text-destructive">{error}</p>{/if}

  {#if preview}
    <div class="flex flex-col gap-3 rounded-md border p-3">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <p class="text-sm font-medium">{preview.rows.length}件をインポートします</p>
        <div class="flex gap-2">
          <Button variant="outline" disabled={importing} onclick={resetPreview}>キャンセル</Button>
          <Button disabled={importing} onclick={importPreview}>
            {#if importing}<Spinner data-icon="inline-start" />登録中…{:else}登録する{/if}
          </Button>
        </div>
      </div>
      {#if preview.created_categories.length || paymentMethods(preview).length}
        <div class="flex flex-wrap gap-x-4 gap-y-1 text-sm">
          {#if preview.created_categories.length}<span>
              新規カテゴリ: {preview.created_categories.join('、')}
            </span>{/if}
          {#if paymentMethods(preview).length}<span>
              新規支払方法: {paymentMethods(preview).join('、')}
            </span>{/if}
        </div>
      {/if}
      <div class="max-h-[28rem] min-w-0 overflow-auto rounded-md border">
        <Table.Root>
          <Table.Header>
            <Table.Row>
              {#if kind === 'recurring-expense'}
                <Table.Head>名称</Table.Head><Table.Head>金額</Table.Head><Table.Head>
                  通貨
                </Table.Head><Table.Head>支払日</Table.Head><Table.Head>
                  開始日
                </Table.Head><Table.Head>カテゴリ</Table.Head><Table.Head>
                  支払方法
                </Table.Head><Table.Head>金額変動</Table.Head><Table.Head>備考</Table.Head>
              {:else}
                <Table.Head>日付</Table.Head><Table.Head>金額</Table.Head><Table.Head>
                  カテゴリ
                </Table.Head>
                {#if kind === 'expense'}<Table.Head>支払方法</Table.Head>{/if}<Table.Head>
                  メモ
                </Table.Head>
              {/if}
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each preview.rows as row, index (index)}
              <Table.Row>
                {#if kind === 'recurring-expense'}
                  <Table.Cell>{recurringRow(row).name}</Table.Cell><Table.Cell>
                    {recurringRow(row).amount}{recurringRow(row).foreign_amount
                      ? ` / ${recurringRow(row).foreign_amount}`
                      : ''}
                  </Table.Cell><Table.Cell>
                    {recurringRow(row).currency || 'JPY'}
                  </Table.Cell><Table.Cell>{recurringRow(row).payment_day}日</Table.Cell><Table.Cell
                  >
                    {recurringRow(row).start_date}
                  </Table.Cell>
                  <Table.Cell>
                    {recurringRow(row).category}{#if recurringRow(row).category_is_new}<Badge
                        variant="secondary"
                      >
                        新規
                      </Badge>{/if}
                  </Table.Cell>
                  <Table.Cell>
                    {recurringRow(row)
                      .payment_method}{#if recurringRow(row).payment_method_is_new}<Badge
                        variant="secondary"
                      >
                        新規
                      </Badge>{/if}
                  </Table.Cell><Table.Cell>
                    {recurringRow(row).is_variable || 'なし'}
                  </Table.Cell><Table.Cell>{recurringRow(row).description || ''}</Table.Cell>
                {:else}
                  <Table.Cell>{expenseRow(row).transaction_date}</Table.Cell><Table.Cell>
                    {expenseRow(row).amount}
                  </Table.Cell>
                  <Table.Cell>
                    {expenseRow(row).category}{#if expenseRow(row).category_is_new}<Badge
                        variant="secondary"
                      >
                        新規
                      </Badge>{/if}
                  </Table.Cell>
                  {#if kind === 'expense'}<Table.Cell>
                      {expenseRow(row)
                        .payment_method}{#if expenseRow(row).payment_method_is_new}<Badge
                          variant="secondary"
                        >
                          新規
                        </Badge>{/if}
                    </Table.Cell>{/if}
                  <Table.Cell>{expenseRow(row).description || ''}</Table.Cell>
                {/if}
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      </div>
    </div>
  {/if}
</div>
