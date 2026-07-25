<script lang="ts">
  import { onMount } from 'svelte';
  import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
  import { toast } from 'svelte-sonner';
  import * as Alert from '$lib/components/ui/alert';
  import { Button } from '$lib/components/ui/button';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import Header from '$lib/components/Header.svelte';
  import PeriodSelector, { type SummaryView } from '$lib/components/PeriodSelector.svelte';
  import { inPeriod, type Transaction } from '$lib/domain/summaries';
  import AnnualSummary from '$lib/features/annual/AnnualSummary.svelte';
  import RecurringExpenseForm from '$lib/features/forms/RecurringExpenseForm.svelte';
  import TransactionForm from '$lib/features/forms/TransactionForm.svelte';
  import LedgerSheet from '$lib/features/ledger/LedgerSheet.svelte';
  import MonthlySummary from '$lib/features/monthly/MonthlySummary.svelte';
  import { api } from '$lib/api';
  import { currentMonth } from '$lib/format';
  import type {
    Expense,
    ExpenseCategory,
    ExpenseInput,
    Income,
    IncomeCategory,
    IncomeInput,
    PaymentMethod,
    RecurringExpense,
    RecurringExpenseInput,
  } from '$lib/types';

  let expenses = $state.raw<Expense[]>([]);
  let incomes = $state.raw<Income[]>([]);
  let expenseCategories = $state.raw<ExpenseCategory[]>([]);
  let incomeCategories = $state.raw<IncomeCategory[]>([]);
  let paymentMethods = $state.raw<PaymentMethod[]>([]);
  let recurringExpenses = $state.raw<RecurringExpense[]>([]);
  let selectedMonth = $state(currentMonth());
  let selectedYear = $state(currentMonth().slice(0, 4));
  let view = $state<SummaryView>('monthly');
  let transactionFormOpen = $state(false);
  let editingExpense = $state<Expense | null>(null);
  let recurringFormOpen = $state(false);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');

  const monthExpenses = $derived(inPeriod(expenses, selectedMonth));
  const monthIncomes = $derived(inPeriod(incomes, selectedMonth));
  const monthLabel = $derived(
    new Intl.DateTimeFormat('ja-JP', { year: 'numeric', month: 'long' }).format(
      new Date(`${selectedMonth}-01T00:00:00`),
    ),
  );
  const availableYears = $derived(
    [
      ...new Set(
        [...expenses, ...incomes]
          .map((item) => item.transaction_date.slice(0, 4))
          .concat(currentMonth().slice(0, 4)),
      ),
    ]
      .sort()
      .reverse(),
  );

  onMount(loadAll);

  async function loadAll() {
    loading = true;
    error = '';
    try {
      const [
        expenseData,
        incomeData,
        expenseCategoryData,
        incomeCategoryData,
        paymentData,
        recurringData,
      ] = await Promise.all([
        api.expenses(),
        api.incomes(),
        api.expenseCategories(),
        api.incomeCategories(),
        api.paymentMethods(),
        api.recurringExpenses(),
      ]);
      expenses = expenseData;
      incomes = incomeData.items;
      expenseCategories = expenseCategoryData.items;
      incomeCategories = incomeCategoryData.items;
      paymentMethods = paymentData.items;
      recurringExpenses = recurringData;
    } catch (caught) {
      error = message(caught, 'データを読み込めませんでした。');
    } finally {
      loading = false;
    }
  }

  async function saveTransaction(
    kind: 'expense' | 'income',
    input: ExpenseInput | IncomeInput,
    keepOpen = false,
  ) {
    saving = true;
    error = '';
    try {
      if (kind === 'expense' && editingExpense) {
        const updated = await api.updateExpense(editingExpense.id, input as ExpenseInput);
        expenses = expenses.map((item) => (item.id === updated.id ? updated : item));
        toast.success('支出を更新しました。');
      } else if (kind === 'expense') {
        expenses = [await api.createExpense(input as ExpenseInput), ...expenses];
        toast.success('支出を登録しました。');
      } else {
        incomes = [await api.createIncome(input as IncomeInput), ...incomes];
        toast.success('収入を登録しました。');
      }
      if (!keepOpen) {
        transactionFormOpen = false;
        editingExpense = null;
      }
      return true;
    } catch (caught) {
      error = message(caught, '登録できませんでした。');
      return false;
    } finally {
      saving = false;
    }
  }

  function openTransaction() {
    editingExpense = null;
    transactionFormOpen = true;
  }

  function editExpense(expense: Expense) {
    editingExpense = expense;
    transactionFormOpen = true;
  }

  function closeTransaction() {
    transactionFormOpen = false;
    editingExpense = null;
  }

  async function saveRecurring(input: RecurringExpenseInput) {
    saving = true;
    error = '';
    try {
      recurringExpenses = [await api.createRecurringExpense(input), ...recurringExpenses];
      recurringFormOpen = false;
      toast.success('固定費を登録しました。');
    } catch (caught) {
      error = message(caught, '固定費を登録できませんでした。');
    } finally {
      saving = false;
    }
  }

  async function removeTransaction(item: Transaction) {
    if (!confirm('この明細を削除しますか？')) return;
    try {
      if (item.kind === 'expense') {
        await api.deleteExpense(item.id);
        expenses = expenses.filter((row) => row.id !== item.id);
      } else {
        await api.deleteIncome(item.id);
        incomes = incomes.filter((row) => row.id !== item.id);
      }
      toast.success('明細を削除しました。');
    } catch (caught) {
      error = message(caught, '削除できませんでした。');
    }
  }

  async function removeExpense(item: Expense) {
    if (!confirm('この明細を削除しますか？')) return;
    try {
      await api.deleteExpense(item.id);
      expenses = expenses.filter((row) => row.id !== item.id);
      toast.success('明細を削除しました。');
    } catch (caught) {
      error = message(caught, '削除できませんでした。');
    }
  }

  function message(caught: unknown, fallback: string) {
    return caught instanceof Error ? caught.message : fallback;
  }
</script>

<svelte:head>
  <title>ホーム | Kakeibo</title>
  <meta name="description" content="毎日の収支をシンプルに管理する家計簿" />
</svelte:head>

<div class="min-h-screen bg-background">
  <Header onTransaction={openTransaction} onRecurring={() => (recurringFormOpen = true)} />
  <main class="mx-auto max-w-7xl px-5 py-8 lg:px-10 lg:py-12">
    <PeriodSelector
      {view}
      month={selectedMonth}
      year={selectedYear}
      years={availableYears}
      onchange={(value) => (view = value)}
      onmonth={(value) => (selectedMonth = value)}
      onyear={(value) => (selectedYear = value)}
    />
    {#if error}
      <Alert.Root variant="destructive" class="mb-6">
        <CircleAlertIcon />
        <Alert.Title>読み込みエラー</Alert.Title>
        <Alert.Description>{error}</Alert.Description>
        <Alert.Action
          ><Button variant="outline" size="sm" onclick={loadAll}>再試行</Button></Alert.Action
        >
      </Alert.Root>
    {/if}
    {#if loading}<div class="grid min-h-72 gap-4 py-12">
        <Skeleton class="h-28 w-full" />
        <Skeleton class="h-28 w-full" />
      </div>
    {:else if view === 'annual'}<AnnualSummary
        year={selectedYear}
        {expenses}
        {incomes}
        categories={expenseCategories}
      />
    {:else if view === 'daily'}<LedgerSheet
        month={selectedMonth}
        {monthLabel}
        expenses={monthExpenses}
        incomes={monthIncomes}
        {expenseCategories}
        {incomeCategories}
        {paymentMethods}
        {recurringExpenses}
        onedit={editExpense}
        ondelete={removeExpense}
      />
    {:else}<MonthlySummary
        {monthLabel}
        expenses={monthExpenses}
        incomes={monthIncomes}
        {expenseCategories}
        {incomeCategories}
        {paymentMethods}
        {recurringExpenses}
        ondelete={removeTransaction}
      />{/if}
  </main>
</div>

{#if transactionFormOpen}<TransactionForm
    {expenseCategories}
    {incomeCategories}
    {paymentMethods}
    {saving}
    initialExpense={editingExpense ?? undefined}
    initialDate={`${selectedMonth}-${String(Math.min(new Date().getDate(), 28)).padStart(2, '0')}`}
    onclose={closeTransaction}
    onsubmit={saveTransaction}
  />{/if}
{#if recurringFormOpen}<RecurringExpenseForm
    categories={expenseCategories}
    {paymentMethods}
    {saving}
    onclose={() => (recurringFormOpen = false)}
    onsubmit={saveRecurring}
  />{/if}
