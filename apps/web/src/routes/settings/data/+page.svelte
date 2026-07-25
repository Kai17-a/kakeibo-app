<script lang="ts">
  import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
  import DownloadIcon from '@lucide/svelte/icons/download';
  import UploadIcon from '@lucide/svelte/icons/upload';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import Header from '$lib/components/Header.svelte';
  import CsvImport from '$lib/features/settings/CsvImport.svelte';
</script>

<svelte:head>
  <title>データ管理 | Kakeibo</title>
  <meta name="description" content="収支データのインポート・エクスポート" />
</svelte:head>

<div class="min-h-screen bg-background">
  <Header />
  <main class="mx-auto flex max-w-5xl flex-col gap-6 px-5 py-8 lg:px-10 lg:py-12">
    <div>
      <Button variant="ghost" size="sm" href="/settings">
        <ArrowLeftIcon data-icon="inline-start" />設定に戻る
      </Button>
    </div>

    <div class="flex flex-col gap-2">
      <p class="text-xs font-semibold tracking-widest text-muted-foreground uppercase">Settings</p>
      <h1 class="font-serif text-3xl font-bold tracking-tight sm:text-4xl">データ管理</h1>
      <p class="text-muted-foreground">収支データのインポート・エクスポートができます。</p>
    </div>

    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <DownloadIcon />データのエクスポート
        </Card.Title>
        <Card.Description>
          登録済みの収支データをCSVファイル（UTF-8・BOM付き）としてダウンロードできます。
        </Card.Description>
      </Card.Header>
      <Card.Content class="flex flex-col gap-3 sm:flex-row">
        <Button variant="outline" href="/api/export/expenses">
          <DownloadIcon data-icon="inline-start" />支出データ（CSV）
        </Button>
        <Button variant="outline" href="/api/export/incomes">
          <DownloadIcon data-icon="inline-start" />収入データ（CSV）
        </Button>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <UploadIcon />データのインポート
        </Card.Title>
        <Card.Description>
          エクスポートしたCSVをそのまま取り込めます（支出: 日付,金額,カテゴリ,支払方法,メモ / 収入:
          日付,金額,カテゴリ,メモ）。未登録のカテゴリや支払方法は自動的に追加されます。同じファイルを再度取り込むと重複して登録されるためご注意ください。
        </Card.Description>
      </Card.Header>
      <Card.Content class="flex flex-col gap-4 sm:flex-row sm:gap-8">
        <CsvImport kind="expense" />
        <CsvImport kind="income" />
      </Card.Content>
    </Card.Root>
  </main>
</div>
