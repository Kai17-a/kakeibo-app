<script lang="ts">
  import { onMount } from 'svelte';
  import { resolve } from '$app/paths';
  import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
  import CreditCardIcon from '@lucide/svelte/icons/credit-card';
  import DatabaseIcon from '@lucide/svelte/icons/database';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import RepeatIcon from '@lucide/svelte/icons/repeat';
  import TagsIcon from '@lucide/svelte/icons/tags';
  import WebhookIcon from '@lucide/svelte/icons/webhook';
  import { toast } from 'svelte-sonner';
  import { SvelteMap } from 'svelte/reactivity';
  import * as Alert from '$lib/components/ui/alert';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import * as Empty from '$lib/components/ui/empty';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import * as Table from '$lib/components/ui/table';
  import * as Tabs from '$lib/components/ui/tabs';
  import Header from '$lib/components/Header.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import CategoryForm from '$lib/features/settings/CategoryForm.svelte';
  import PaymentMethodForm from '$lib/features/settings/PaymentMethodForm.svelte';
  import RecurringExpenseForm from '$lib/features/forms/RecurringExpenseForm.svelte';
  import RecurringIncomeForm from '$lib/features/forms/RecurringIncomeForm.svelte';
  import WebhookUrlForm from '$lib/features/settings/WebhookUrlForm.svelte';
  import { api } from '$lib/api';
  import { formatYen } from '$lib/format';
  import type {
    CategoryInput,
    ExpenseCategory,
    IncomeCategory,
    NamedResource,
    PaymentMethod,
    PaymentMethodInput,
    RecurringExpense,
    RecurringExpenseInput,
    RecurringIncome,
    RecurringIncomeInput,
    WebhookUrl,
    WebhookUrlInput,
  } from '$lib/types';

  type CategoryKind = 'expense' | 'income';
  type SettingsTab = CategoryKind | 'payment' | 'recurring' | 'recurring-income' | 'webhook';

  let expenseCategories = $state.raw<ExpenseCategory[]>([]);
  let incomeCategories = $state.raw<IncomeCategory[]>([]);
  let paymentMethods = $state.raw<PaymentMethod[]>([]);
  let recurringExpenses = $state.raw<RecurringExpense[]>([]);
  let recurringIncomes = $state.raw<RecurringIncome[]>([]);
  let webhookUrls = $state.raw<WebhookUrl[]>([]);
  let activeKind = $state<SettingsTab>('expense');
  let formOpen = $state(false);
  let editingCategory = $state<ExpenseCategory | IncomeCategory | null>(null);
  let paymentFormOpen = $state(false);
  let editingPaymentMethod = $state<PaymentMethod | null>(null);
  let recurringFormOpen = $state(false);
  let editingRecurring = $state<RecurringExpense | null>(null);
  let recurringIncomeFormOpen = $state(false);
  let editingRecurringIncome = $state<RecurringIncome | null>(null);
  let webhookFormOpen = $state(false);
  let editingWebhookUrl = $state<WebhookUrl | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');

  onMount(loadAll);

  async function loadAll() {
    loading = true;
    error = '';
    try {
      const [
        expenseData,
        incomeData,
        paymentData,
        recurringData,
        recurringIncomeData,
        webhookData,
      ] = await Promise.all([
        api.expenseCategories(),
        api.incomeCategories(),
        api.paymentMethods(),
        api.recurringExpenses(),
        api.recurringIncomes(),
        api.webhookUrls(),
      ]);
      expenseCategories = expenseData.items;
      incomeCategories = incomeData.items;
      paymentMethods = paymentData.items;
      recurringExpenses = recurringData;
      recurringIncomes = recurringIncomeData;
      webhookUrls = webhookData;
    } catch (caught) {
      error = message(caught, 'データを読み込めませんでした。');
    } finally {
      loading = false;
    }
  }

  async function saveCategory(input: CategoryInput) {
    saving = true;
    error = '';
    const editing = editingCategory;
    try {
      if (editing) {
        if (activeKind === 'income') {
          const category = await api.updateIncomeCategory(editing.id, input);
          incomeCategories = incomeCategories
            .map((item) => (item.id === category.id ? category : item))
            .sort(compareByName);
        } else {
          const category = await api.updateExpenseCategory(editing.id, input);
          expenseCategories = expenseCategories
            .map((item) => (item.id === category.id ? category : item))
            .sort(compareByName);
        }
        toast.success(`${activeKind === 'income' ? '収入' : '支出'}カテゴリを更新しました。`);
      } else if (activeKind === 'income') {
        const category = await api.createIncomeCategory(input);
        incomeCategories = [...incomeCategories, category].sort(compareByName);
        toast.success('収入カテゴリを追加しました。');
      } else {
        const category = await api.createExpenseCategory(input);
        expenseCategories = [...expenseCategories, category].sort(compareByName);
        toast.success('支出カテゴリを追加しました。');
      }
      closeCategoryForm();
    } catch (caught) {
      error = message(
        caught,
        editing ? 'カテゴリを更新できませんでした。' : 'カテゴリを追加できませんでした。',
      );
    } finally {
      saving = false;
    }
  }

  function openCategoryForm() {
    editingCategory = null;
    formOpen = true;
  }

  function editCategory(category: ExpenseCategory | IncomeCategory) {
    editingCategory = category;
    formOpen = true;
  }

  function closeCategoryForm() {
    formOpen = false;
    editingCategory = null;
  }

  async function removeCategory(category: ExpenseCategory | IncomeCategory) {
    try {
      if (activeKind === 'income') {
        await api.deleteIncomeCategory(category.id);
        incomeCategories = incomeCategories.filter((item) => item.id !== category.id);
      } else {
        await api.deleteExpenseCategory(category.id);
        expenseCategories = expenseCategories.filter((item) => item.id !== category.id);
      }
      toast.success('カテゴリを削除しました。');
    } catch {
      error =
        'カテゴリを削除できませんでした。登録済みの明細で使用されているか、子カテゴリが存在するため削除できません。';
    }
  }

  function openPaymentForm() {
    editingPaymentMethod = null;
    paymentFormOpen = true;
  }

  function editPaymentMethod(item: PaymentMethod) {
    editingPaymentMethod = item;
    paymentFormOpen = true;
  }

  function closePaymentForm() {
    paymentFormOpen = false;
    editingPaymentMethod = null;
  }

  async function savePaymentMethod(input: PaymentMethodInput) {
    saving = true;
    error = '';
    const editing = editingPaymentMethod;
    try {
      if (editing) {
        const updated = await api.updatePaymentMethod(editing.id, input);
        paymentMethods = paymentMethods
          .map((item) => (item.id === updated.id ? updated : item))
          .sort(compareByName);
        toast.success('支払方法を更新しました。');
      } else {
        paymentMethods = [...paymentMethods, await api.createPaymentMethod(input)].sort(
          compareByName,
        );
        toast.success('支払方法を追加しました。');
      }
      closePaymentForm();
    } catch (caught) {
      error = message(
        caught,
        editing ? '支払方法を更新できませんでした。' : '支払方法を追加できませんでした。',
      );
    } finally {
      saving = false;
    }
  }

  async function removePaymentMethod(item: PaymentMethod) {
    try {
      await api.deletePaymentMethod(item.id);
      paymentMethods = paymentMethods.filter((row) => row.id !== item.id);
      toast.success('支払方法を削除しました。');
    } catch {
      error =
        '支払方法を削除できませんでした。登録済みの明細で使用されている場合は削除できません。';
    }
  }

  const expenseCategoryNames = $derived(
    new SvelteMap(expenseCategories.map((item) => [item.id, item.name])),
  );
  const paymentMethodNames = $derived(
    new SvelteMap(paymentMethods.map((item) => [item.id, item.name])),
  );

  async function saveRecurring(input: RecurringExpenseInput) {
    saving = true;
    error = '';
    const editing = editingRecurring;
    try {
      if (editing) {
        const updated = await api.updateRecurringExpense(editing.id, input);
        recurringExpenses = recurringExpenses.map((item) =>
          item.id === updated.id ? updated : item,
        );
        toast.success('定期支出を更新しました。');
      } else {
        recurringExpenses = [await api.createRecurringExpense(input), ...recurringExpenses];
        toast.success('定期支出を登録しました。');
      }
      closeRecurringForm();
    } catch (caught) {
      error = message(
        caught,
        editing ? '定期支出を更新できませんでした。' : '定期支出を登録できませんでした。',
      );
    } finally {
      saving = false;
    }
  }

  function openRecurringForm() {
    editingRecurring = null;
    recurringFormOpen = true;
  }

  function editRecurring(item: RecurringExpense) {
    editingRecurring = item;
    recurringFormOpen = true;
  }

  function closeRecurringForm() {
    recurringFormOpen = false;
    editingRecurring = null;
  }

  async function removeRecurringExpense(item: RecurringExpense) {
    try {
      await api.deleteRecurringExpense(item.id);
      recurringExpenses = recurringExpenses.filter((row) => row.id !== item.id);
      toast.success('定期支出を削除しました。');
    } catch {
      error =
        '定期支出を削除できませんでした。登録済みの明細で使用されている場合は削除できません。';
    }
  }

  async function saveRecurringIncome(input: RecurringIncomeInput) {
    saving = true;
    error = '';
    const editing = editingRecurringIncome;
    try {
      if (editing) {
        const updated = await api.updateRecurringIncome(editing.id, input);
        recurringIncomes = recurringIncomes.map((item) =>
          item.id === updated.id ? updated : item,
        );
        toast.success('定期収入を更新しました。');
      } else {
        recurringIncomes = [await api.createRecurringIncome(input), ...recurringIncomes];
        toast.success('定期収入を登録しました。');
      }
      closeRecurringIncomeForm();
    } catch (caught) {
      error = message(
        caught,
        editing ? '定期収入を更新できませんでした。' : '定期収入を登録できませんでした。',
      );
    } finally {
      saving = false;
    }
  }

  function openRecurringIncomeForm() {
    editingRecurringIncome = null;
    recurringIncomeFormOpen = true;
  }
  function editRecurringIncome(item: RecurringIncome) {
    editingRecurringIncome = item;
    recurringIncomeFormOpen = true;
  }
  function closeRecurringIncomeForm() {
    recurringIncomeFormOpen = false;
    editingRecurringIncome = null;
  }
  async function removeRecurringIncome(item: RecurringIncome) {
    try {
      await api.deleteRecurringIncome(item.id);
      recurringIncomes = recurringIncomes.filter((row) => row.id !== item.id);
      toast.success('定期収入を削除しました。');
    } catch {
      error =
        '定期収入を削除できませんでした。登録済みの明細で使用されている場合は削除できません。';
    }
  }

  async function saveWebhookUrl(input: WebhookUrlInput) {
    saving = true;
    error = '';
    const editing = editingWebhookUrl;
    try {
      if (editing) {
        const updated = await api.updateWebhookUrl(editing.id, input);
        webhookUrls = webhookUrls.map((item) => (item.id === updated.id ? updated : item));
        toast.success('Webhook URLを更新しました。');
      } else {
        webhookUrls = [await api.createWebhookUrl(input), ...webhookUrls];
        toast.success('Webhook URLを登録しました。');
      }
      closeWebhookForm();
    } catch (caught) {
      error = message(
        caught,
        editing ? 'Webhook URLを更新できませんでした。' : 'Webhook URLを登録できませんでした。',
      );
    } finally {
      saving = false;
    }
  }

  function openWebhookForm() {
    editingWebhookUrl = null;
    webhookFormOpen = true;
  }

  function editWebhookUrl(item: WebhookUrl) {
    editingWebhookUrl = item;
    webhookFormOpen = true;
  }

  function closeWebhookForm() {
    webhookFormOpen = false;
    editingWebhookUrl = null;
  }

  async function removeWebhookUrl(item: WebhookUrl) {
    try {
      await api.deleteWebhookUrl(item.id);
      webhookUrls = webhookUrls.filter((row) => row.id !== item.id);
      toast.success('Webhook URLを削除しました。');
    } catch {
      error = 'Webhook URLを削除できませんでした。';
    }
  }

  type DeleteTarget =
    | { type: 'category'; item: ExpenseCategory | IncomeCategory }
    | { type: 'payment'; item: PaymentMethod }
    | { type: 'recurring'; item: RecurringExpense }
    | { type: 'recurring-income'; item: RecurringIncome }
    | { type: 'webhook'; item: WebhookUrl };

  let deleteTarget = $state<DeleteTarget | null>(null);

  const deleteDescription = $derived.by(() => {
    const target = deleteTarget;
    if (!target) return '';
    if (target.type === 'category') return `カテゴリ「${target.item.name}」を削除しますか？`;
    if (target.type === 'payment') return `支払方法「${target.item.name}」を削除しますか？`;
    if (target.type === 'recurring') return `定期支出「${target.item.name}」を削除しますか？`;
    if (target.type === 'recurring-income')
      return `定期収入「${target.item.name}」を削除しますか？`;
    return `Webhook URL「${target.item.url}」を削除しますか？`;
  });

  function askDeleteCategory(category: ExpenseCategory | IncomeCategory) {
    deleteTarget = { type: 'category', item: category };
  }

  function askDeletePaymentMethod(item: PaymentMethod) {
    deleteTarget = { type: 'payment', item };
  }

  function askDeleteRecurring(item: RecurringExpense) {
    deleteTarget = { type: 'recurring', item };
  }
  function askDeleteRecurringIncome(item: RecurringIncome) {
    deleteTarget = { type: 'recurring-income', item };
  }

  function askDeleteWebhookUrl(item: WebhookUrl) {
    deleteTarget = { type: 'webhook', item };
  }

  function cancelDelete() {
    deleteTarget = null;
  }

  async function confirmDelete() {
    const target = deleteTarget;
    deleteTarget = null;
    if (!target) return;
    if (target.type === 'category') {
      await removeCategory(target.item);
    } else if (target.type === 'payment') {
      await removePaymentMethod(target.item);
    } else if (target.type === 'recurring') {
      await removeRecurringExpense(target.item);
    } else if (target.type === 'recurring-income') {
      await removeRecurringIncome(target.item);
    } else {
      await removeWebhookUrl(target.item);
    }
  }

  function compareByName(a: NamedResource, b: NamedResource) {
    return a.name.localeCompare(b.name, 'ja');
  }

  function groupCategories<T extends ExpenseCategory | IncomeCategory>(categories: T[]): T[] {
    const children = new SvelteMap<string, T[]>();
    for (const category of categories) {
      if (!category.parent_category_id) continue;
      const siblings = children.get(category.parent_category_id) ?? [];
      siblings.push(category);
      children.set(category.parent_category_id, siblings);
    }
    return categories
      .filter((category) => category.parent_category_id === null)
      .flatMap((parent) => [parent, ...(children.get(parent.id) ?? [])]);
  }

  function message(caught: unknown, fallback: string) {
    return caught instanceof Error ? caught.message : fallback;
  }
</script>

<svelte:head>
  <title>カテゴリ設定 | Kakeibo</title>
  <meta name="description" content="支出と収入のカテゴリを管理" />
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
      <h1 class="font-serif text-3xl font-bold tracking-tight sm:text-4xl">カテゴリ設定</h1>
      <p class="text-muted-foreground">
        収支の登録や集計に使用するカテゴリ・支払方法・定期支出・定期収入・Webhook通知先を管理できます。
      </p>
    </div>

    {#if error}
      <Alert.Root variant="destructive">
        <CircleAlertIcon />
        <Alert.Title>エラー</Alert.Title>
        <Alert.Description>{error}</Alert.Description>
        {#if loading}
          <Alert.Action>
            <Button variant="outline" size="sm" onclick={loadAll}>再試行</Button>
          </Alert.Action>
        {/if}
      </Alert.Root>
    {/if}
    <Tabs.Root bind:value={activeKind} class="w-full">
      <Tabs.List variant="line" class="w-full justify-start sm:w-fit">
        <Tabs.Trigger value="expense">支出カテゴリ</Tabs.Trigger>
        <Tabs.Trigger value="income">収入カテゴリ</Tabs.Trigger>
        <Tabs.Trigger value="payment">支払方法</Tabs.Trigger>
        <Tabs.Trigger value="recurring">定期支出</Tabs.Trigger>
        <Tabs.Trigger value="recurring-income">定期収入</Tabs.Trigger>
        <Tabs.Trigger value="webhook">Webhook</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="expense">
        {@render categoryPanel('支出カテゴリ', expenseCategories)}
      </Tabs.Content>
      <Tabs.Content value="income">
        {@render categoryPanel('収入カテゴリ', incomeCategories)}
      </Tabs.Content>
      <Tabs.Content value="payment">
        {@render paymentMethodPanel()}
      </Tabs.Content>
      <Tabs.Content value="recurring">
        {@render recurringPanel()}
      </Tabs.Content>
      <Tabs.Content value="recurring-income">
        {@render recurringIncomePanel()}
      </Tabs.Content>
      <Tabs.Content value="webhook">
        {@render webhookPanel()}
      </Tabs.Content>
    </Tabs.Root>

    <a
      href={resolve('/settings/data')}
      class="flex items-center gap-4 bg-card p-6 text-card-foreground shadow-sm ring-1 ring-foreground/5 transition-colors hover:bg-muted/50"
    >
      <DatabaseIcon class="size-5 shrink-0 text-muted-foreground" />
      <span class="min-w-0 flex-1">
        <span class="block font-semibold">データ管理</span>
        <span class="block text-muted-foreground">
          収支データのインポート・エクスポートはこちら
        </span>
      </span>
      <ChevronRightIcon class="size-5 shrink-0 text-muted-foreground" />
    </a>
  </main>
</div>

{#snippet categoryPanel(title: string, categories: (ExpenseCategory | IncomeCategory)[])}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        {title}
        <Badge variant="secondary">{categories.length}件</Badge>
      </Card.Title>
      <Card.Description>収支の登録時に選択できる{title}を管理します。</Card.Description>
      <Card.Action>
        <Button onclick={openCategoryForm}>
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
            <Button onclick={openCategoryForm}>
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
              <Table.Head class="w-36 text-right">操作</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each groupCategories(categories) as category (category.id)}
              <Table.Row>
                <Table.Cell class={category.parent_category_id ? 'pl-8' : ''}>
                  <span class="font-medium">{category.name}</span>
                  <span class="ml-2 text-xs text-muted-foreground">
                    {category.parent_category_id ? '子カテゴリ' : '親カテゴリ'}
                  </span>
                </Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {category.description || '説明はありません'}
                </Table.Cell>
                <Table.Cell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`${category.name}を編集`}
                      onclick={() => editCategory(category)}
                    >
                      編集
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      aria-label={`${category.name}を削除`}
                      onclick={() => askDeleteCategory(category)}
                    >
                      削除
                    </Button>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      {/if}
    </Card.Content>
  </Card.Root>
{/snippet}

{#snippet paymentMethodPanel()}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        支払方法
        <Badge variant="secondary">{paymentMethods.length}件</Badge>
      </Card.Title>
      <Card.Description>支出の登録時に選択できる支払方法を管理します。</Card.Description>
      <Card.Action>
        <Button onclick={openPaymentForm}>
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
      {:else if paymentMethods.length === 0}
        <Empty.Root class="min-h-64 border">
          <Empty.Media variant="icon"><CreditCardIcon /></Empty.Media>
          <Empty.Header>
            <Empty.Title>支払方法がありません</Empty.Title>
            <Empty.Description>最初の支払方法を追加してください。</Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <Button onclick={openPaymentForm}>
              <PlusIcon data-icon="inline-start" />支払方法を追加
            </Button>
          </Empty.Content>
        </Empty.Root>
      {:else}
        <Table.Root>
          <Table.Caption>登録済みの支払方法一覧</Table.Caption>
          <Table.Header>
            <Table.Row>
              <Table.Head class="w-1/3">支払方法名</Table.Head>
              <Table.Head>説明</Table.Head>
              <Table.Head class="text-right">残高</Table.Head>
              <Table.Head class="w-36 text-right">操作</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each paymentMethods as method (method.id)}
              <Table.Row>
                <Table.Cell class="font-medium">{method.name}</Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {method.description || '説明はありません'}
                </Table.Cell>
                <Table.Cell class="text-right tabular-nums">
                  {method.balance == null ? '未設定' : formatYen(method.balance)}
                </Table.Cell>
                <Table.Cell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`${method.name}を編集`}
                      onclick={() => editPaymentMethod(method)}
                    >
                      編集
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      aria-label={`${method.name}を削除`}
                      onclick={() => askDeletePaymentMethod(method)}
                    >
                      削除
                    </Button>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      {/if}
    </Card.Content>
  </Card.Root>
{/snippet}
{#snippet recurringPanel()}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        定期支出
        <Badge variant="secondary">{recurringExpenses.length}件</Badge>
      </Card.Title>
      <Card.Description>
        毎月発生する固定費・準固定費（金額変動）の支出予定を管理します。
      </Card.Description>
      <Card.Action>
        <Button onclick={openRecurringForm}>
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
      {:else if recurringExpenses.length === 0}
        <Empty.Root class="min-h-64 border">
          <Empty.Media variant="icon"><RepeatIcon /></Empty.Media>
          <Empty.Header>
            <Empty.Title>定期支出がありません</Empty.Title>
            <Empty.Description>最初の定期支出を追加してください。</Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <Button onclick={openRecurringForm}>
              <PlusIcon data-icon="inline-start" />定期支出を追加
            </Button>
          </Empty.Content>
        </Empty.Root>
      {:else}
        <Table.Root>
          <Table.Caption>登録済みの定期支出一覧</Table.Caption>
          <Table.Header>
            <Table.Row>
              <Table.Head class="w-1/4">名称</Table.Head>
              <Table.Head>カテゴリ / 支払方法</Table.Head>
              <Table.Head>支払日</Table.Head>
              <Table.Head class="text-right">金額</Table.Head>
              <Table.Head>状態</Table.Head>
              <Table.Head class="w-36 text-right">操作</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each recurringExpenses as item (item.id)}
              <Table.Row>
                <Table.Cell class="font-medium">
                  <span class="flex items-center gap-2">
                    {item.name}{#if item.is_variable}<Badge variant="secondary">
                        準固定費
                      </Badge>{/if}
                  </span>
                </Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {expenseCategoryNames.get(item.category_id) ?? ''} / {paymentMethodNames.get(
                    item.payment_method_id,
                  ) ?? ''}
                </Table.Cell>
                <Table.Cell>毎月 {item.payment_day} 日</Table.Cell>
                <Table.Cell class="text-right">{formatYen(item.amount)}</Table.Cell>
                <Table.Cell>
                  {#if item.is_active}<Badge variant="outline">有効</Badge>{:else}<Badge
                      variant="secondary"
                    >
                      無効
                    </Badge>{/if}
                </Table.Cell>
                <Table.Cell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`定期支出 ${item.name}を編集`}
                      onclick={() => editRecurring(item)}
                    >
                      編集
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      aria-label={`定期支出 ${item.name}を削除`}
                      onclick={() => askDeleteRecurring(item)}
                    >
                      削除
                    </Button>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      {/if}
    </Card.Content>
  </Card.Root>
{/snippet}
{#snippet recurringIncomePanel()}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        定期収入 <Badge variant="secondary">{recurringIncomes.length}件</Badge>
      </Card.Title>
      <Card.Description>
        毎月発生する固定収入・準固定収入（金額変動）の入金予定を管理します。
      </Card.Description>
      <Card.Action>
        <Button onclick={openRecurringIncomeForm}><PlusIcon data-icon="inline-start" />追加</Button>
      </Card.Action>
    </Card.Header>
    <Card.Content>
      {#if loading}
        <div class="flex flex-col gap-3">
          <Skeleton class="h-16 w-full" /><Skeleton class="h-16 w-full" /><Skeleton
            class="h-16 w-full"
          />
        </div>
      {:else if recurringIncomes.length === 0}
        <Empty.Root class="min-h-64 border">
          <Empty.Media variant="icon"><RepeatIcon /></Empty.Media>
          <Empty.Header>
            <Empty.Title>定期収入がありません</Empty.Title><Empty.Description>
              最初の定期収入を追加してください。
            </Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <Button onclick={openRecurringIncomeForm}>
              <PlusIcon data-icon="inline-start" />定期収入を追加
            </Button>
          </Empty.Content>
        </Empty.Root>
      {:else}
        <Table.Root>
          <Table.Caption>登録済みの定期収入一覧</Table.Caption>
          <Table.Header>
            <Table.Row>
              <Table.Head class="w-1/4">名称</Table.Head><Table.Head>
                カテゴリ
              </Table.Head><Table.Head>入金日</Table.Head><Table.Head class="text-right">
                金額
              </Table.Head><Table.Head>状態</Table.Head><Table.Head class="w-36 text-right">
                操作
              </Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each recurringIncomes as item (item.id)}
              <Table.Row>
                <Table.Cell class="font-medium">
                  <span class="flex items-center gap-2">
                    {item.name}{#if item.is_variable}<Badge variant="secondary">
                        準固定収入
                      </Badge>{/if}
                  </span>
                </Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {incomeCategories.find((category) => category.id === item.category_id)?.name ??
                    ''}
                </Table.Cell>
                <Table.Cell>毎月 {item.payment_day} 日</Table.Cell>
                <Table.Cell class="text-right">{formatYen(item.amount)}</Table.Cell>
                <Table.Cell>
                  {#if item.is_active}<Badge variant="outline">有効</Badge>{:else}<Badge
                      variant="secondary"
                    >
                      無効
                    </Badge>{/if}
                </Table.Cell>
                <Table.Cell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`定期収入 ${item.name}を編集`}
                      onclick={() => editRecurringIncome(item)}
                    >
                      編集
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      aria-label={`定期収入 ${item.name}を削除`}
                      onclick={() => askDeleteRecurringIncome(item)}
                    >
                      削除
                    </Button>
                  </div>
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      {/if}
    </Card.Content>
  </Card.Root>
{/snippet}
{#snippet webhookPanel()}
  <Card.Root>
    <Card.Header>
      <Card.Title class="flex items-center gap-2">
        Webhook
        <Badge variant="secondary">{webhookUrls.length}件</Badge>
      </Card.Title>
      <Card.Description>
        支出・収入の登録時に通知を送信するWebhook URLを管理します。
      </Card.Description>
      <Card.Action>
        <Button onclick={openWebhookForm}>
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
      {:else if webhookUrls.length === 0}
        <Empty.Root class="min-h-64 border">
          <Empty.Media variant="icon"><WebhookIcon /></Empty.Media>
          <Empty.Header>
            <Empty.Title>Webhook URLがありません</Empty.Title>
            <Empty.Description>通知を送信するURLを追加してください。</Empty.Description>
          </Empty.Header>
          <Empty.Content>
            <Button onclick={openWebhookForm}>
              <PlusIcon data-icon="inline-start" />Webhook URLを追加
            </Button>
          </Empty.Content>
        </Empty.Root>
      {:else}
        <Table.Root>
          <Table.Caption>登録済みのWebhook URL一覧</Table.Caption>
          <Table.Header>
            <Table.Row>
              <Table.Head class="w-1/3">URL</Table.Head>
              <Table.Head>説明</Table.Head>
              <Table.Head>状態</Table.Head>
              <Table.Head class="w-36 text-right">操作</Table.Head>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each webhookUrls as item (item.id)}
              <Table.Row>
                <Table.Cell class="max-w-0 truncate font-medium">{item.url}</Table.Cell>
                <Table.Cell class="text-muted-foreground">
                  {item.description || '説明はありません'}
                </Table.Cell>
                <Table.Cell>
                  {#if item.is_active}<Badge variant="outline">有効</Badge>{:else}<Badge
                      variant="secondary"
                    >
                      無効
                    </Badge>{/if}
                </Table.Cell>
                <Table.Cell class="text-right">
                  <div class="flex justify-end gap-2">
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`${item.url}を編集`}
                      onclick={() => editWebhookUrl(item)}
                    >
                      編集
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      aria-label={`${item.url}を削除`}
                      onclick={() => askDeleteWebhookUrl(item)}
                    >
                      削除
                    </Button>
                  </div>
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
    kind={activeKind === 'income' ? 'income' : 'expense'}
    {saving}
    categories={activeKind === 'income' ? incomeCategories : expenseCategories}
    initial={editingCategory ?? undefined}
    onclose={closeCategoryForm}
    onsubmit={saveCategory}
  />
{/if}
{#if paymentFormOpen}
  <PaymentMethodForm
    {saving}
    initial={editingPaymentMethod ?? undefined}
    onclose={closePaymentForm}
    onsubmit={savePaymentMethod}
  />
{/if}
{#if recurringFormOpen}
  <RecurringExpenseForm
    categories={expenseCategories}
    {paymentMethods}
    {saving}
    initial={editingRecurring ?? undefined}
    onclose={closeRecurringForm}
    onsubmit={saveRecurring}
  />
{/if}
{#if recurringIncomeFormOpen}
  <RecurringIncomeForm
    categories={incomeCategories}
    {saving}
    initial={editingRecurringIncome ?? undefined}
    onclose={closeRecurringIncomeForm}
    onsubmit={saveRecurringIncome}
  />
{/if}
{#if webhookFormOpen}
  <WebhookUrlForm
    {saving}
    initial={editingWebhookUrl ?? undefined}
    onclose={closeWebhookForm}
    onsubmit={saveWebhookUrl}
  />
{/if}
<ConfirmDialog
  open={deleteTarget !== null}
  title="削除の確認"
  description={deleteDescription}
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
/>
