<script lang="ts">
  import { onMount } from 'svelte';
  import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
  import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
  import DownloadIcon from '@lucide/svelte/icons/download';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import TagsIcon from '@lucide/svelte/icons/tags';
  import { toast } from 'svelte-sonner';
  import * as Alert from '$lib/components/ui/alert';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Empty from '$lib/components/ui/empty';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import * as Table from '$lib/components/ui/table';
  import * as Tabs from '$lib/components/ui/tabs';
  import Header from '$lib/components/Header.svelte';
  import CategoryForm from '$lib/features/settings/CategoryForm.svelte';
  import { api } from '$lib/api';
  import type { CategoryInput, ExpenseCategory, IncomeCategory, NamedResource } from '$lib/types';

  type CategoryKind = 'expense' | 'income';

  let expenseCategories = $state.raw<ExpenseCategory[]>([]);
  let incomeCategories = $state.raw<IncomeCategory[]>([]);
  let activeKind = $state<CategoryKind>('expense');
  let formOpen = $state(false);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');

  onMount(loadCategories);

  async function loadCategories() {
    loading = true;
    error = '';
    try {
      const [expenseData, incomeData] = await Promise.all([
        api.expenseCategories(),
        api.incomeCategories(),
      ]);
      expenseCategories = expenseData.items;
      incomeCategories = incomeData.items;
    } catch (caught) {
      error = message(caught, 'カテゴリを読み込めませんでした。');
    } finally {
      loading = false;
    }
  }

  async function saveCategory(input: CategoryInput) {
    saving = true;
    error = '';
    try {
      if (activeKind === 'expense') {
        const category = await api.createExpenseCategory(input);
        expenseCategories = [...expenseCategories, category].sort(compareByName);
      } else {
        const category = await api.createIncomeCategory(input);
        incomeCategories = [...incomeCategories, category].sort(compareByName);
      }
      formOpen = false;
      toast.success(`${activeKind === 'expense' ? '支出' : '収入'}カテゴリを追加しました。`);
    } catch (caught) {
      error = message(caught, 'カテゴリを追加できませんでした。');
    } finally {
      saving = false;
    }
  }

  function compareByName(a: NamedResource, b: NamedResource) {
    return a.name.localeCompare(b.name, 'ja');
  }

  function message(caught: unknown, fallback: string) {
    return caught instanceof Error ? caught.message : fallback;
  }
</script>

<svelte:head>
  <title>設定 | Kakeibo</title>
  <meta name="description" content="収支のカテゴリ管理とデータのエクスポート" />
</svelte:head>

<div class="min-h-screen bg-background">
  <Header />
  <main class="mx-auto flex max-w-5xl flex-col gap-6 px-5 py-8 lg:px-10 lg:py-12">
    <div>
      <Button variant="ghost" size="sm" href="/">
        <ArrowLeftIcon data-icon="inline-start" />ホームに戻る
      </Button>
    </div>

    <div class="flex flex-col gap-2">
      <p class="text-xs font-semibold tracking-widest text-muted-foreground uppercase">Settings</p>
      <h1 class="font-serif text-3xl font-bold tracking-tight sm:text-4xl">設定</h1>
      <p class="text-muted-foreground">
        収支の登録や集計に使用するカテゴリの管理と、データのエクスポートができます。
      </p>
    </div>

    {#if error}
      <Alert.Root variant="destructive">
        <CircleAlertIcon />
        <Alert.Title>エラー</Alert.Title>
        <Alert.Description>{error}</Alert.Description>
        {#if loading}
          <Alert.Action>
            <Button variant="outline" size="sm" onclick={loadCategories}>再試行</Button>
          </Alert.Action>
        {/if}
      </Alert.Root>
    {/if}
    <Tabs.Root bind:value={activeKind} class="w-full">
      <Tabs.List variant="line" class="w-full justify-start sm:w-fit">
        <Tabs.Trigger value="expense">支出カテゴリ</Tabs.Trigger>
        <Tabs.Trigger value="income">収入カテゴリ</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="expense">
        {@render categoryPanel('支出カテゴリ', expenseCategories)}
      </Tabs.Content>
      <Tabs.Content value="income">
        {@render categoryPanel('収入カテゴリ', incomeCategories)}
      </Tabs.Content>
    </Tabs.Root>

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
  </main>
</div>

{#snippet categoryPanel(title: string, categories: NamedResource[])}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        {title}
        <Badge variant="secondary">{categories.length}件</Badge>
      </Card.Title>
      <Card.Description>収支の登録時に選択できる{title}を管理します。</Card.Description>
      <Card.Action>
        <Button onclick={() => (formOpen = true)}>
          <PlusIcon data-icon="inline-start" />追加
        </Button>
      </Card.Action>
    </Card.Header>
    <Card.Content>
      {#if loading}
        <div class="flex flex-col gap-3">
          <Skeleton class="h-16 w-full" />
          <Skeleton class="h-16 w-full" />
          <Skeleton class="h-16 w-full" />
        </div>
      {:else if categories.length === 0}
        <Empty.Root class="min-h-64 border">
          <Empty.Media variant="icon"><TagsIcon /></Empty.Media>
          <Empty.Header>
            <Empty.Title>カテゴリがありません</Empty.Title>
            <Empty.Description>最初のカテゴリを追加してください。</Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <Button onclick={() => (formOpen = true)}>
              <PlusIcon data-icon="inline-start" />カテゴリを追加
            </Button>
          </Empty.Content>
        </Empty.Root>
      {:else}
        <Table.Root>
          <Table.Caption>登録済みの{title}一覧</Table.Caption>
          <Table.Header>
            <Table.Row>
              <Table.Head class="w-1/3">カテゴリ名</Table.Head>
              <Table.Head>説明</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each categories as category (category.id)}
              <Table.Row>
                <Table.Cell class="font-medium">{category.name}</Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {category.description || '説明はありません'}
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      {/if}
    </Card.Content>
  </Card.Root>
{/snippet}

{#if formOpen}
  <CategoryForm
    kind={activeKind}
    {saving}
    onclose={() => (formOpen = false)}
    onsubmit={saveCategory}
  />
{/if}
