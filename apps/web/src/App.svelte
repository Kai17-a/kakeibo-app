<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import { api } from "./lib/api";
  import { currentMonth, formatDate, formatYen } from "./lib/format";
  import type {
    Expense,
    ExpenseCategory,
    Income,
    IncomeCategory,
    PaymentMethod,
    RecurringExpense,
  } from "./lib/types";

  type Transaction = (Expense | Income) & { kind: "expense" | "income" };
  type FormKind = "expense" | "income" | null;
  type SummaryView = "monthly" | "daily" | "annual";
  type DailySheetTab = "ledger" | "categories";

  let expenses = $state.raw<Expense[]>([]);
  let incomes = $state.raw<Income[]>([]);
  let expenseCategories = $state.raw<ExpenseCategory[]>([]);
  let incomeCategories = $state.raw<IncomeCategory[]>([]);
  let paymentMethods = $state.raw<PaymentMethod[]>([]);
  let recurringExpenses = $state.raw<RecurringExpense[]>([]);
  let selectedMonth = $state(currentMonth());
  let selectedYear = $state(currentMonth().slice(0, 4));
  let summaryView = $state<SummaryView>("monthly");
  let dailySheetTab = $state<DailySheetTab>("ledger");
  let loading = $state(true);
  let saving = $state(false);
  let error = $state("");
  let notice = $state("");
  let formKind = $state<FormKind>(null);
  let recurringFormOpen = $state(false);

  let transactionDate = $state(new Date().toISOString().slice(0, 10));
  let amount = $state("");
  let categoryId = $state("");
  let paymentMethodId = $state("");
  let description = $state("");
  let recurringName = $state("");
  let recurringAmount = $state("");
  let recurringPaymentDay = $state(1);
  let recurringStartDate = $state(new Date().toISOString().slice(0, 10));
  let recurringEndDate = $state("");
  let recurringCategoryId = $state("");
  let recurringPaymentMethodId = $state("");
  let recurringDescription = $state("");
  let recurringIsActive = $state(true);

  const expenseCategoryNames = $derived(
    new SvelteMap(expenseCategories.map((item) => [item.id, item.name])),
  );
  const incomeCategoryNames = $derived(
    new SvelteMap(incomeCategories.map((item) => [item.id, item.name])),
  );
  const paymentMethodNames = $derived(
    new SvelteMap(paymentMethods.map((item) => [item.id, item.name])),
  );
  const monthLabel = $derived(
    new Intl.DateTimeFormat("ja-JP", { year: "numeric", month: "long" }).format(
      new Date(`${selectedMonth}-01T00:00:00`),
    ),
  );
  const monthExpenses = $derived(
    expenses.filter((item) => item.transaction_date.startsWith(selectedMonth)),
  );
  const monthIncomes = $derived(
    incomes.filter((item) => item.transaction_date.startsWith(selectedMonth)),
  );
  const expenseTotal = $derived(
    monthExpenses.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const incomeTotal = $derived(
    monthIncomes.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const balance = $derived(incomeTotal - expenseTotal);
  const transactions = $derived.by(() =>
    [
      ...monthExpenses.map((item) => ({ ...item, kind: "expense" as const })),
      ...monthIncomes.map((item) => ({ ...item, kind: "income" as const })),
    ].sort((a, b) => b.transaction_date.localeCompare(a.transaction_date)),
  );
  const categorySpending = $derived.by(() => {
    const totals = new SvelteMap<string, number>();
    for (const item of monthExpenses)
      totals.set(
        item.category_id,
        (totals.get(item.category_id) ?? 0) + Number(item.amount),
      );
    return [...totals.entries()]
      .map(([id, total]) => ({
        id,
        name: expenseCategoryNames.get(id) ?? "未分類",
        total,
      }))
      .sort((a, b) => b.total - a.total);
  });
  const activeMonthlyCategories = $derived(expenseCategories);
  const dailyCategorySummary = $derived.by(() => {
    const daysInMonth = new Date(
      Number(selectedMonth.slice(0, 4)),
      Number(selectedMonth.slice(5, 7)),
      0,
    ).getDate();
    return Array.from({ length: daysInMonth }, (_, index) => {
      const date = `${selectedMonth}-${String(index + 1).padStart(2, "0")}`;
      const values = new SvelteMap<string, number>();
      for (const item of monthExpenses.filter((expense) => expense.transaction_date === date)) {
        values.set(item.category_id, (values.get(item.category_id) ?? 0) + Number(item.amount));
      }
      return {
        date,
        values,
        total: [...values.values()].reduce((sum, value) => sum + value, 0),
      };
    });
  });
  const monthlyIncomeBreakdown = $derived(
    incomeCategories.map((category) => ({
      id: category.id,
      name: category.name,
      total: monthIncomes
        .filter((item) => item.category_id === category.id)
        .reduce((sum, item) => sum + Number(item.amount), 0),
    })),
  );
  const monthlyPaymentBreakdown = $derived(
    paymentMethods.map((method) => ({
      id: method.id,
      name: method.name,
      total: monthExpenses
        .filter((item) => item.payment_method_id === method.id)
        .reduce((sum, item) => sum + Number(item.amount), 0),
    })),
  );
  const monthlyFixedExpenses = $derived(
    monthExpenses.filter((item) => item.recurring_expense_id !== null),
  );
  const monthlyFixedTotal = $derived(
    monthlyFixedExpenses.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const monthlyRecurringExpenses = $derived(
    recurringExpenses.filter((item) =>
      item.is_active &&
      item.start_date.slice(0, 7) <= selectedMonth &&
      (!item.end_date || item.end_date.slice(0, 7) >= selectedMonth),
    ),
  );
  const monthlyRecurringTotal = $derived(
    monthlyRecurringExpenses.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const monthlyVariableExpenses = $derived(
    monthExpenses.filter((item) => item.recurring_expense_id === null),
  );
  const monthlyVariableCategorySpending = $derived.by(() => {
    const totals = new SvelteMap<string, number>();
    for (const item of monthlyVariableExpenses)
      totals.set(item.category_id, (totals.get(item.category_id) ?? 0) + Number(item.amount));
    return expenseCategories.map((category) => ({
      id: category.id,
      name: category.name,
      total: totals.get(category.id) ?? 0,
    }));
  });
  const monthlyExpenseLedger = $derived(
    [...monthExpenses].sort((a, b) =>
      a.transaction_date.localeCompare(b.transaction_date) || a.created_at.localeCompare(b.created_at),
    ),
  );
  const annualExpenses = $derived(
    expenses.filter((item) => item.transaction_date.startsWith(selectedYear)),
  );
  const annualIncomes = $derived(
    incomes.filter((item) => item.transaction_date.startsWith(selectedYear)),
  );
  const annualExpenseTotal = $derived(
    annualExpenses.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const annualIncomeTotal = $derived(
    annualIncomes.reduce((sum, item) => sum + Number(item.amount), 0),
  );
  const annualBalance = $derived(annualIncomeTotal - annualExpenseTotal);
  const annualMonthlySummary = $derived.by(() =>
    Array.from({ length: 12 }, (_, index) => {
      const month = `${selectedYear}-${String(index + 1).padStart(2, "0")}`;
      const income = annualIncomes
        .filter((item) => item.transaction_date.startsWith(month))
        .reduce((sum, item) => sum + Number(item.amount), 0);
      const expense = annualExpenses
        .filter((item) => item.transaction_date.startsWith(month))
        .reduce((sum, item) => sum + Number(item.amount), 0);
      return { month: index + 1, income, expense, balance: income - expense };
    }),
  );
  const annualChartMax = $derived(
    Math.max(1, ...annualMonthlySummary.flatMap((item) => [item.income, item.expense])),
  );
  const annualCategorySpending = $derived.by(() => {
    const totals = new SvelteMap<string, number>();
    for (const item of annualExpenses)
      totals.set(item.category_id, (totals.get(item.category_id) ?? 0) + Number(item.amount));
    return [...totals.entries()]
      .map(([id, total]) => ({ id, name: expenseCategoryNames.get(id) ?? "未分類", total }))
      .sort((a, b) => b.total - a.total);
  });
  const availableYears = $derived.by(() => {
    const years = new SvelteSet(
      [...expenses, ...incomes].map((item) => item.transaction_date.slice(0, 4)),
    );
    years.add(currentMonth().slice(0, 4));
    return [...years].sort().reverse();
  });

  onMount(loadAll);

  async function loadAll() {
    loading = true;
    error = "";
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
      error =
        caught instanceof Error
          ? caught.message
          : "データを読み込めませんでした。";
    } finally {
      loading = false;
    }
  }

  function openForm(kind: Exclude<FormKind, null>) {
    formKind = kind;
    transactionDate = `${selectedMonth}-${String(Math.min(new Date().getDate(), 28)).padStart(2, "0")}`;
    amount = "";
    description = "";
    categoryId =
      kind === "expense"
        ? (expenseCategories[0]?.id ?? "")
        : (incomeCategories[0]?.id ?? "");
    paymentMethodId = paymentMethods[0]?.id ?? "";
  }

  function openRecurringForm() {
    recurringName = "";
    recurringAmount = "";
    recurringPaymentDay = 1;
    recurringStartDate = new Date().toISOString().slice(0, 10);
    recurringEndDate = "";
    recurringCategoryId = expenseCategories[0]?.id ?? "";
    recurringPaymentMethodId = paymentMethods[0]?.id ?? "";
    recurringDescription = "";
    recurringIsActive = true;
    recurringFormOpen = true;
  }

  async function submitRecurringExpense(event: SubmitEvent) {
    event.preventDefault();
    if (!recurringName || !recurringAmount || !recurringCategoryId || !recurringPaymentMethodId) return;
    saving = true;
    error = "";
    try {
      const created = await api.createRecurringExpense({
        name: recurringName,
        amount: recurringAmount,
        payment_day: recurringPaymentDay,
        start_date: recurringStartDate,
        end_date: recurringEndDate || null,
        category_id: recurringCategoryId,
        payment_method_id: recurringPaymentMethodId,
        is_active: recurringIsActive,
        description: recurringDescription || null,
      });
      recurringExpenses = [created, ...recurringExpenses];
      recurringFormOpen = false;
      notice = "固定費を登録しました。";
    } catch (caught) {
      error = caught instanceof Error ? caught.message : "固定費を登録できませんでした。";
    } finally {
      saving = false;
    }
  }

  async function submitTransaction(event: SubmitEvent) {
    event.preventDefault();
    if (
      !formKind ||
      !amount ||
      !categoryId ||
      (formKind === "expense" && !paymentMethodId)
    )
      return;
    saving = true;
    error = "";
    try {
      if (formKind === "expense") {
        const created = await api.createExpense({
          transaction_date: transactionDate,
          amount,
          category_id: categoryId,
          payment_method_id: paymentMethodId,
          recurring_expense_id: null,
          description: description || null,
        });
        expenses = [created, ...expenses];
      } else {
        const created = await api.createIncome({
          transaction_date: transactionDate,
          amount,
          category_id: categoryId,
          description: description || null,
        });
        incomes = [created, ...incomes];
      }
      notice = `${formKind === "expense" ? "支出" : "収入"}を登録しました。`;
      formKind = null;
    } catch (caught) {
      error =
        caught instanceof Error ? caught.message : "登録できませんでした。";
    } finally {
      saving = false;
    }
  }

  async function removeTransaction(item: Transaction) {
    if (!confirm("この明細を削除しますか？")) return;
    try {
      if (item.kind === "expense") {
        await api.deleteExpense(item.id);
        expenses = expenses.filter((row) => row.id !== item.id);
      } else {
        await api.deleteIncome(item.id);
        incomes = incomes.filter((row) => row.id !== item.id);
      }
      notice = "明細を削除しました。";
    } catch (caught) {
      error =
        caught instanceof Error ? caught.message : "削除できませんでした。";
    }
  }

  function transactionLabel(item: Transaction) {
    return (
      item.description ||
      (item.kind === "expense"
        ? expenseCategoryNames.get(item.category_id)
        : incomeCategoryNames.get(item.category_id)) ||
      "名称なし"
    );
  }
</script>

<svelte:head>
  <title>ホーム | Kakeibo</title>
  <meta name="description" content="毎日の収支をシンプルに管理する家計簿" />
</svelte:head>

<div class="min-h-screen bg-[#f4f1e9]">
  <header class="border-b border-[#d8d5cc] bg-[#f8f6f0]/95">
    <div
      class="mx-auto flex max-w-7xl items-center justify-between px-5 py-4 lg:px-10"
    >
      <a class="flex items-center gap-3" href="/" aria-label="Kakeibo ホーム">
        <span
          class="grid size-10 place-items-center rounded-xl bg-[#245c4a] text-lg font-black text-white"
        >
          K
        </span>
        <strong class="font-serif text-xl tracking-tight">Kakeibo</strong>
      </a>
      <div class="flex items-center gap-2">
        <button
          class="rounded-xl border border-[#c8c7bf] bg-white px-4 py-2 text-sm font-semibold hover:bg-[#eeece5]"
          onclick={openRecurringForm}>＋ 固定費</button
        >
        <button
          class="rounded-xl border border-[#c8c7bf] bg-white px-4 py-2 text-sm font-semibold hover:bg-[#eeece5]"
          onclick={() => openForm("income")}>＋ 収入</button
        >
        <button
          class="rounded-xl bg-[#245c4a] px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-[#194738]"
          onclick={() => openForm("expense")}>＋ 支出</button
        >
      </div>
    </div>
  </header>

  <main class="mx-auto max-w-7xl px-5 py-8 lg:px-10 lg:py-12">
    <section class="mb-8 flex flex-wrap items-end justify-between gap-5">
      <div>
        <p class="mb-1 text-sm font-bold tracking-widest text-[#39705d]">
          OVERVIEW
        </p>
        <h1 class="font-serif text-3xl font-semibold lg:text-4xl">
          {summaryView === "annual" ? `${selectedYear}年の家計` : summaryView === "daily" ? "日別カテゴリ集計" : "今月の家計"}
        </h1>
      </div>
      <div class="flex flex-wrap items-end gap-3">
        <div class="flex rounded-xl border border-[#cbc9c0] bg-white p-1" aria-label="集計期間">
          <button class={["rounded-lg px-4 py-2 text-sm font-bold", summaryView === "monthly" ? "bg-[#245c4a] text-white" : "text-[#68746e] hover:bg-[#f0eee7]"]} aria-pressed={summaryView === "monthly"} onclick={() => summaryView = "monthly"}>月間</button>
          <button class={["rounded-lg px-4 py-2 text-sm font-bold", summaryView === "daily" ? "bg-[#245c4a] text-white" : "text-[#68746e] hover:bg-[#f0eee7]"]} aria-pressed={summaryView === "daily"} onclick={() => summaryView = "daily"}>日別集計</button>
          <button class={["rounded-lg px-4 py-2 text-sm font-bold", summaryView === "annual" ? "bg-[#245c4a] text-white" : "text-[#68746e] hover:bg-[#f0eee7]"]} aria-pressed={summaryView === "annual"} onclick={() => summaryView = "annual"}>年間</button>
        </div>
        {#if summaryView !== "annual"}
          <label class="grid gap-1.5 text-xs font-bold text-[#59665f]">表示する月<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2 text-sm shadow-sm outline-none focus:border-[#39705d]" type="month" bind:value={selectedMonth} /></label>
        {:else}
          <label class="grid gap-1.5 text-xs font-bold text-[#59665f]">表示する年<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2 text-sm shadow-sm outline-none focus:border-[#39705d]" bind:value={selectedYear}>{#each availableYears as year (year)}<option value={year}>{year}年</option>{/each}</select></label>
        {/if}
      </div>
    </section>

    {#if error}<div
        class="mb-6 flex items-center justify-between rounded-xl border border-[#dfaaa1] bg-[#fff2ef] px-4 py-3 text-sm text-[#8c3025]"
        role="alert"
      >
        <span>{error}</span><button
          class="font-bold"
          onclick={() => {
            error = "";
            loadAll();
          }}>再試行</button
        >
      </div>{/if}
    {#if notice}<div
        class="mb-6 rounded-xl border border-[#a8cab9] bg-[#edf8f2] px-4 py-3 text-sm text-[#245c4a]"
        role="status"
      >
        {notice}
      </div>{/if}

    {#if loading}
      <div class="grid min-h-72 place-items-center">
        <p class="animate-pulse text-[#65736c]">
          家計データを読み込んでいます…
        </p>
      </div>
    {:else if summaryView === "daily"}
      <div class="overflow-hidden rounded-2xl border border-[#cfcfc8] bg-white shadow-[0_5px_20px_rgba(38,51,45,.04)]">
        <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[#d9d9d2] bg-[#eef3ed] px-5 py-3"><div><h2 class="font-serif text-xl font-semibold">{monthLabel} 家計簿</h2><p class="mt-0.5 text-xs text-[#6c7771]">月次収支・明細・日別カテゴリ集計</p></div><span class="rounded-md border border-[#c8d1c9] bg-white px-3 py-1 text-xs font-bold text-[#52635a]">単位：円</span></div>

        <nav class="flex gap-1 border-b border-[#d9d9d2] bg-[#faf9f5] px-4 pt-3" aria-label="家計簿シート">
          <button class={["rounded-t-lg border border-b-0 px-5 py-2.5 text-sm font-bold", dailySheetTab === "ledger" ? "border-[#c9cec9] bg-white text-[#245c4a]" : "border-transparent text-[#707a74] hover:bg-[#f0eee7]"]} aria-current={dailySheetTab === "ledger" ? "page" : undefined} onclick={() => dailySheetTab = "ledger"}>収支・明細</button>
          <button class={["rounded-t-lg border border-b-0 px-5 py-2.5 text-sm font-bold", dailySheetTab === "categories" ? "border-[#c9cec9] bg-white text-[#245c4a]" : "border-transparent text-[#707a74] hover:bg-[#f0eee7]"]} aria-current={dailySheetTab === "categories" ? "page" : undefined} onclick={() => dailySheetTab = "categories"}>日ごとのカテゴリ別支出</button>
        </nav>

        {#if dailySheetTab === "ledger"}
        <div class="grid xl:grid-cols-[18rem_31rem_minmax(46rem,1fr)]">
          <aside class="grid content-start border-b border-[#d9d9d2] xl:border-b-0 xl:border-r">
            <section class="border-b border-[#d9d9d2] p-4"><h3 class="mb-2 bg-[#dce9df] px-3 py-2 text-sm font-bold">収支サマリー</h3><dl class="grid grid-cols-2 text-sm"><dt class="border-b px-3 py-2">収入</dt><dd class="border-b px-3 py-2 text-right font-bold tabular-nums text-[#245c4a]">{formatYen(incomeTotal)}</dd><dt class="border-b px-3 py-2">支出</dt><dd class="border-b px-3 py-2 text-right font-bold tabular-nums text-[#b45845]">{formatYen(expenseTotal)}</dd><dt class="px-3 py-2 font-bold">収支</dt><dd class={['px-3 py-2 text-right font-bold tabular-nums', balance < 0 ? 'text-[#b45845]' : 'text-[#245c4a]']}>{formatYen(balance)}</dd></dl></section>

            <section class="border-b border-[#d9d9d2] p-4"><h3 class="mb-2 bg-[#e8ece7] px-3 py-2 text-sm font-bold">収入</h3><dl class="grid grid-cols-2 text-sm">{#each monthlyIncomeBreakdown as item (item.id)}<dt class="border-b border-[#eeeeea] px-3 py-2">{item.name}</dt><dd class="border-b border-[#eeeeea] px-3 py-2 text-right tabular-nums">{item.total.toLocaleString("ja-JP")}</dd>{/each}<dt class="px-3 py-2 font-bold">合計</dt><dd class="px-3 py-2 text-right font-bold tabular-nums">{incomeTotal.toLocaleString("ja-JP")}</dd></dl></section>

            <section class="p-4"><h3 class="mb-2 bg-[#e8ece7] px-3 py-2 text-sm font-bold">支払種別</h3><dl class="grid grid-cols-2 text-sm">{#each monthlyPaymentBreakdown as item (item.id)}<dt class="border-b border-[#eeeeea] px-3 py-2">{item.name}</dt><dd class="border-b border-[#eeeeea] px-3 py-2 text-right tabular-nums">{item.total.toLocaleString("ja-JP")}</dd>{/each}</dl></section>
          </aside>

          <div class="grid content-start border-b border-[#d9d9d2] xl:border-b-0 xl:border-r">
            <section class="border-b border-[#d9d9d2] p-4"><div class="mb-2 flex items-center justify-between bg-[#e6eadf] px-3 py-2"><h3 class="text-sm font-bold">支出（固定費）</h3><button class="text-xs font-bold text-[#245c4a] hover:underline" onclick={openRecurringForm}>＋ 登録</button></div><dl class="grid grid-cols-2 text-sm">{#each monthlyRecurringExpenses as item (item.id)}<dt class="border-b border-[#eeeeea] px-3 py-2"><span class="block">{item.name}</span><small class="text-[#838b86]">毎月{item.payment_day}日</small></dt><dd class="border-b border-[#eeeeea] px-3 py-2 text-right tabular-nums">{Number(item.amount).toLocaleString("ja-JP")}</dd>{/each}{#if !monthlyRecurringExpenses.length}<dt class="col-span-2 px-3 py-3 text-[#8a918d]">登録された固定費はありません</dt>{/if}<dt class="px-3 py-2 font-bold">合計</dt><dd class="px-3 py-2 text-right font-bold tabular-nums">{monthlyRecurringTotal.toLocaleString("ja-JP")}</dd></dl></section>

            <section class="p-4"><h3 class="mb-2 bg-[#f0e8dc] px-3 py-2 text-sm font-bold">支出（変動費）</h3><dl class="grid grid-cols-2 text-sm">{#each monthlyVariableCategorySpending as item (item.id)}<dt class="border-b border-[#eeeeea] px-3 py-2">{item.name}</dt><dd class="border-b border-[#eeeeea] px-3 py-2 text-right tabular-nums">{item.total.toLocaleString("ja-JP")}</dd>{/each}<dt class="px-3 py-2 font-bold">合計</dt><dd class="px-3 py-2 text-right font-bold tabular-nums">{(expenseTotal - monthlyFixedTotal).toLocaleString("ja-JP")}</dd></dl></section>
          </div>

          <section class="min-w-0 p-4"><h3 class="mb-2 bg-[#e8ece7] px-3 py-2 text-sm font-bold">支出明細</h3><div class="max-h-[38rem] overflow-auto"><table class="w-full min-w-[620px] text-sm"><thead class="sticky top-0 bg-[#f5f5f1] text-xs"><tr><th class="px-3 py-2 text-left">日付</th><th class="px-3 py-2 text-left">摘要</th><th class="px-3 py-2 text-left">支払種別</th><th class="px-3 py-2 text-right">金額</th><th class="px-3 py-2 text-left">備考</th></tr></thead><tbody class="divide-y divide-[#eeeeea]">{#each monthlyExpenseLedger as item (item.id)}<tr class="hover:bg-[#faf9f5]"><td class="whitespace-nowrap px-3 py-2">{formatDate(item.transaction_date)}</td><td class="px-3 py-2 font-semibold">{expenseCategoryNames.get(item.category_id) ?? "未分類"}</td><td class="px-3 py-2">{paymentMethodNames.get(item.payment_method_id) ?? "—"}</td><td class="px-3 py-2 text-right tabular-nums">{Number(item.amount).toLocaleString("ja-JP")}</td><td class="max-w-44 truncate px-3 py-2 text-[#6d7772]" title={item.description ?? ""}>{item.description ?? ""}</td></tr>{/each}</tbody></table>{#if !monthlyExpenseLedger.length}<p class="py-12 text-center text-sm text-[#8a918d]">この月の明細はありません</p>{/if}</div></section>
        </div>
        {:else}
          <section class="p-4"><h3 class="mb-2 bg-[#dce9df] px-3 py-2 text-sm font-bold">日ごとのカテゴリ別支出</h3>{#if activeMonthlyCategories.length}<div class="overflow-x-auto"><table class="w-full min-w-[760px] border-collapse text-sm"><thead class="bg-[#f5f5f1] text-xs"><tr><th class="sticky left-0 z-10 min-w-28 border-b border-r bg-[#f5f5f1] px-3 py-2 text-left">日付</th>{#each activeMonthlyCategories as category (category.id)}<th class="min-w-24 border-b px-3 py-2 text-right">{category.name}</th>{/each}<th class="sticky right-0 min-w-28 border-b border-l bg-[#e9efe9] px-3 py-2 text-right">合計</th></tr></thead><tbody class="divide-y divide-[#eeeeea]">{#each dailyCategorySummary as day (day.date)}<tr class={day.total ? "hover:bg-[#faf9f5]" : "text-[#a2a7a4]"}><th class="sticky left-0 border-r bg-white px-3 py-2 text-left font-semibold">{formatDate(day.date)}</th>{#each activeMonthlyCategories as category (category.id)}<td class="px-3 py-2 text-right tabular-nums">{day.values.get(category.id)?.toLocaleString("ja-JP") ?? ""}</td>{/each}<td class="sticky right-0 border-l bg-[#f1f5f1] px-3 py-2 text-right font-bold tabular-nums">{day.total ? day.total.toLocaleString("ja-JP") : ""}</td></tr>{/each}</tbody><tfoot class="border-t-2 bg-[#e9efe9]"><tr><th class="sticky left-0 border-r bg-[#e9efe9] px-3 py-3 text-left">合計</th>{#each activeMonthlyCategories as category (category.id)}<td class="px-3 py-3 text-right font-bold tabular-nums">{monthExpenses.filter((item) => item.category_id === category.id).reduce((sum, item) => sum + Number(item.amount), 0).toLocaleString("ja-JP")}</td>{/each}<td class="sticky right-0 border-l bg-[#dce8de] px-3 py-3 text-right font-bold tabular-nums">{expenseTotal.toLocaleString("ja-JP")}</td></tr></tfoot></table></div>{:else}<p class="py-12 text-center text-sm text-[#8a918d]">支出を登録すると日別・カテゴリ別に表示されます。</p>{/if}</section>
        {/if}
      </div>
    {:else if summaryView === "annual"}
      <section class="grid gap-4 md:grid-cols-3" aria-label={`${selectedYear}年の収支概要`}>
        <article class="rounded-2xl border border-[#d8d5cc] bg-white p-6 shadow-[0_5px_20px_rgba(38,51,45,.04)]"><p class="text-sm font-semibold text-[#6d7872]">年間収入</p><p class="mt-3 font-serif text-3xl font-semibold text-[#245c4a]">{formatYen(annualIncomeTotal)}</p><p class="mt-4 text-xs text-[#8a918d]">月平均 {formatYen(annualIncomeTotal / 12)}</p></article>
        <article class="rounded-2xl border border-[#d8d5cc] bg-white p-6 shadow-[0_5px_20px_rgba(38,51,45,.04)]"><p class="text-sm font-semibold text-[#6d7872]">年間支出</p><p class="mt-3 font-serif text-3xl font-semibold text-[#b45845]">{formatYen(annualExpenseTotal)}</p><p class="mt-4 text-xs text-[#8a918d]">月平均 {formatYen(annualExpenseTotal / 12)}</p></article>
        <article class="rounded-2xl bg-[#203e34] p-6 text-white shadow-lg"><p class="text-sm font-semibold text-[#bcd3c9]">年間収支</p><p class="mt-3 font-serif text-3xl font-semibold">{annualBalance >= 0 ? "+" : ""}{formatYen(annualBalance)}</p><p class="mt-4 text-xs text-[#bcd3c9]">貯蓄率 {annualIncomeTotal ? Math.round(annualBalance / annualIncomeTotal * 100) : 0}%</p></article>
      </section>

      <div class="mt-8 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
        <section class="rounded-2xl border border-[#d8d5cc] bg-white p-5 shadow-[0_5px_20px_rgba(38,51,45,.04)] lg:p-6">
          <div class="flex flex-wrap items-start justify-between gap-3"><div><h2 class="font-serif text-xl font-semibold">月別の収支推移</h2><p class="mt-1 text-xs text-[#7b847f]">{selectedYear}年 1月〜12月</p></div><div class="flex gap-4 text-xs font-semibold text-[#68746e]"><span class="flex items-center gap-1.5"><i class="size-2.5 rounded-sm bg-[#7aac96]"></i>収入</span><span class="flex items-center gap-1.5"><i class="size-2.5 rounded-sm bg-[#d29a62]"></i>支出</span></div></div>
          <div class="mt-7 flex h-72 items-end gap-2 border-b border-[#d8d5cc] px-1 sm:gap-3" aria-label="月別収支グラフ">
            {#each annualMonthlySummary as item (item.month)}
              <div class="group flex h-full min-w-0 flex-1 flex-col justify-end">
                <div class="relative flex h-[calc(100%_-_2rem)] items-end justify-center gap-0.5 sm:gap-1">
                  <div class="w-2.5 rounded-t-sm bg-[#7aac96] transition-all group-hover:bg-[#51826d] sm:w-4" style:height={`${item.income / annualChartMax * 100}%`} title={`${item.month}月 収入 ${formatYen(item.income)}`}></div>
                  <div class="w-2.5 rounded-t-sm bg-[#d29a62] transition-all group-hover:bg-[#b87844] sm:w-4" style:height={`${item.expense / annualChartMax * 100}%`} title={`${item.month}月 支出 ${formatYen(item.expense)}`}></div>
                </div>
                <span class="mt-2 text-center text-[10px] font-semibold text-[#727c76] sm:text-xs">{item.month}月</span>
              </div>
            {/each}
          </div>
        </section>

        <section class="rounded-2xl border border-[#d8d5cc] bg-white p-5 shadow-[0_5px_20px_rgba(38,51,45,.04)]">
          <h2 class="font-serif text-xl font-semibold">年間支出の内訳</h2>
          <p class="mt-1 text-xs text-[#7b847f]">カテゴリ別の合計</p>
          {#if annualCategorySpending.length}
            <div class="mt-6 grid gap-4">{#each annualCategorySpending.slice(0, 6) as category, index (category.id)}<div><div class="mb-1.5 flex justify-between gap-3 text-sm"><span class="truncate font-semibold"><span class="mr-2 text-[#a7ada9]">{index + 1}</span>{category.name}</span><span class="shrink-0 tabular-nums text-[#59655f]">{formatYen(category.total)}</span></div><div class="h-2 overflow-hidden rounded-full bg-[#edeae2]"><div class="h-full rounded-full bg-[#d29a62]" style:width={`${annualExpenseTotal ? category.total / annualExpenseTotal * 100 : 0}%`}></div></div></div>{/each}</div>
          {:else}<div class="grid min-h-52 place-items-center text-center"><p class="text-sm text-[#7d8681]">この年の支出はまだありません。</p></div>{/if}
        </section>
      </div>

      <section class="mt-6 overflow-hidden rounded-2xl border border-[#d8d5cc] bg-white shadow-[0_5px_20px_rgba(38,51,45,.04)]">
        <div class="border-b border-[#e5e2da] px-5 py-4"><h2 class="font-serif text-xl font-semibold">月ごとの収支</h2></div>
        <div class="overflow-x-auto"><table class="w-full min-w-[600px] text-sm"><thead class="bg-[#f7f5ef] text-left text-xs text-[#6c7771]"><tr><th class="px-5 py-3 font-semibold">月</th><th class="px-5 py-3 text-right font-semibold">収入</th><th class="px-5 py-3 text-right font-semibold">支出</th><th class="px-5 py-3 text-right font-semibold">収支</th></tr></thead><tbody class="divide-y divide-[#eeece6]">{#each annualMonthlySummary as item (item.month)}<tr class="hover:bg-[#faf9f5]"><td class="px-5 py-3 font-bold">{item.month}月</td><td class="px-5 py-3 text-right tabular-nums text-[#245c4a]">{formatYen(item.income)}</td><td class="px-5 py-3 text-right tabular-nums">{formatYen(item.expense)}</td><td class={['px-5 py-3 text-right font-bold tabular-nums', item.balance < 0 ? 'text-[#b45845]' : 'text-[#245c4a]']}>{item.balance > 0 ? '+' : ''}{formatYen(item.balance)}</td></tr>{/each}</tbody></table></div>
      </section>
    {:else}
      <section
        class="grid gap-4 md:grid-cols-3"
        aria-label={`${monthLabel}の収支概要`}
      >
        <article
          class="rounded-2xl border border-[#d8d5cc] bg-white p-6 shadow-[0_5px_20px_rgba(38,51,45,.04)]"
        >
          <p class="text-sm font-semibold text-[#6d7872]">収入</p>
          <p class="mt-3 font-serif text-3xl font-semibold text-[#245c4a]">
            {formatYen(incomeTotal)}
          </p>
          <p class="mt-4 text-xs text-[#8a918d]">
            {monthIncomes.length} 件の入金
          </p>
        </article>
        <article
          class="rounded-2xl border border-[#d8d5cc] bg-white p-6 shadow-[0_5px_20px_rgba(38,51,45,.04)]"
        >
          <p class="text-sm font-semibold text-[#6d7872]">支出</p>
          <p class="mt-3 font-serif text-3xl font-semibold text-[#b45845]">
            {formatYen(expenseTotal)}
          </p>
          <p class="mt-4 text-xs text-[#8a918d]">
            {monthExpenses.length} 件の支払い
          </p>
        </article>
        <article class="rounded-2xl bg-[#203e34] p-6 text-white shadow-lg">
          <p class="text-sm font-semibold text-[#bcd3c9]">残り</p>
          <p class="mt-3 font-serif text-3xl font-semibold">
            {formatYen(balance)}
          </p>
          <div class="mt-4 h-1.5 overflow-hidden rounded-full bg-white/15">
            <div
              class="h-full rounded-full bg-[#e4c476]"
              style:width={`${incomeTotal ? Math.min(100, Math.max(0, (balance / incomeTotal) * 100)) : 0}%`}
            ></div>
          </div>
        </article>
      </section>

      <div class="mt-8 grid gap-6 lg:grid-cols-[1.6fr_1fr]">
        <section
          class="overflow-hidden rounded-2xl border border-[#d8d5cc] bg-white shadow-[0_5px_20px_rgba(38,51,45,.04)]"
        >
          <div
            class="flex items-center justify-between border-b border-[#e5e2da] px-5 py-4"
          >
            <div>
              <h2 class="font-serif text-xl font-semibold">最近の明細</h2>
              <p class="mt-0.5 text-xs text-[#7b847f]">{monthLabel}</p>
            </div>
            <span
              class="rounded-full bg-[#f0eee7] px-3 py-1 text-xs font-bold text-[#66716b]"
              >{transactions.length} 件</span
            >
          </div>
          {#if transactions.length}
            <ul class="divide-y divide-[#eeece6]">
              {#each transactions as item (item.id)}
                <li
                  class="group flex items-center gap-3 px-5 py-4 hover:bg-[#faf9f5]"
                >
                  <span
                    class={[
                      "grid size-10 shrink-0 place-items-center rounded-xl text-lg",
                      item.kind === "income"
                        ? "bg-[#e4f0ea] text-[#245c4a]"
                        : "bg-[#f6e9e5] text-[#ad4f3d]",
                    ]}>{item.kind === "income" ? "↘" : "↗"}</span
                  >
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-bold">
                      {transactionLabel(item)}
                    </p>
                    <p class="mt-0.5 truncate text-xs text-[#7d8681]">
                      {formatDate(item.transaction_date)}{item.kind ===
                        "expense" &&
                      paymentMethodNames.get(item.payment_method_id)
                        ? ` · ${paymentMethodNames.get(item.payment_method_id)}`
                        : ""}
                    </p>
                  </div>
                  <p
                    class={[
                      "font-serif text-base font-semibold tabular-nums",
                      item.kind === "income"
                        ? "text-[#245c4a]"
                        : "text-[#29342f]",
                    ]}
                  >
                    {item.kind === "income" ? "+" : "−"}{formatYen(item.amount)}
                  </p>
                  <button
                    class="ml-1 rounded-lg px-2 py-1 text-sm text-[#9a837d] opacity-0 hover:bg-[#f4e5e1] hover:text-[#9a3f31] focus:opacity-100 group-hover:opacity-100"
                    aria-label={`${transactionLabel(item)}を削除`}
                    onclick={() => removeTransaction(item)}>削除</button
                  >
                </li>
              {/each}
            </ul>
          {:else}<div class="grid min-h-64 place-items-center px-6 text-center">
              <div>
                <p class="text-3xl">☕</p>
                <p class="mt-3 font-semibold">この月の明細はまだありません</p>
                <p class="mt-1 text-sm text-[#7d8681]">
                  右上のボタンから最初の記録を追加しましょう。
                </p>
              </div>
            </div>{/if}
        </section>

        <div class="grid content-start gap-6">
          <section
            class="rounded-2xl border border-[#d8d5cc] bg-white p-5 shadow-[0_5px_20px_rgba(38,51,45,.04)]"
          >
            <h2 class="font-serif text-xl font-semibold">支出の内訳</h2>
            {#if categorySpending.length}<div class="mt-5 grid gap-4">
                {#each categorySpending as category (category.id)}<div>
                    <div class="mb-1.5 flex justify-between text-sm">
                      <span class="font-semibold">{category.name}</span><span
                        class="tabular-nums text-[#59655f]"
                        >{formatYen(category.total)}</span
                      >
                    </div>
                    <div class="h-2 overflow-hidden rounded-full bg-[#edeae2]">
                      <div
                        class="h-full rounded-full bg-[#d29a62]"
                        style:width={`${expenseTotal ? (category.total / expenseTotal) * 100 : 0}%`}
                      ></div>
                    </div>
                  </div>{/each}
              </div>{:else}<p class="mt-4 text-sm text-[#7d8681]">
                支出を登録すると内訳が表示されます。
              </p>{/if}
          </section>
          <section class="rounded-2xl border border-[#d8d5cc] bg-[#e9eee8] p-5">
            <div class="flex items-center justify-between">
              <h2 class="font-serif text-xl font-semibold">定期支出</h2>
              <span class="text-xs font-bold text-[#607269]"
                >{recurringExpenses.filter((item) => item.is_active).length} 件</span
              >
            </div>
            <ul class="mt-3 divide-y divide-[#cfd8d1]">
              {#each recurringExpenses.filter((item) => item.is_active) as item (item.id)}<li
                  class="flex items-center justify-between py-3 text-sm"
                >
                  <span
                    ><strong class="block">{item.name}</strong><small
                      class="text-[#68766f]">毎月 {item.payment_day} 日</small
                    ></span
                  ><span class="font-serif font-semibold"
                    >{formatYen(item.amount)}</span
                  >
                </li>{/each}
            </ul>
            {#if !recurringExpenses.some((item) => item.is_active)}<p
                class="mt-3 text-sm text-[#68766f]"
              >
                有効な定期支出はありません。
              </p>{/if}
          </section>
        </div>
      </div>
    {/if}
  </main>
</div>

{#if formKind}
  <div
    class="fixed inset-0 z-50 grid place-items-center bg-[#15241e]/55 p-4"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) formKind = null;
    }}
  >
    <div
      class="w-full max-w-lg rounded-2xl bg-[#fbfaf6] p-6 shadow-2xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="form-title"
    >
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs font-bold tracking-widest text-[#39705d]">
            NEW ENTRY
          </p>
          <h2 class="mt-1 font-serif text-2xl font-semibold" id="form-title">
            {formKind === "expense" ? "支出" : "収入"}を登録
          </h2>
        </div>
        <button
          class="grid size-9 place-items-center rounded-full hover:bg-[#eae7df]"
          aria-label="閉じる"
          onclick={() => (formKind = null)}>✕</button
        >
      </div>
      <form class="mt-6 grid gap-4" onsubmit={submitTransaction}>
        <label class="grid gap-1.5 text-sm font-semibold">
          日付
          <input
            class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]"
            type="date"
            required
            bind:value={transactionDate}
          />
        </label>
        <label class="grid gap-1.5 text-sm font-semibold">
          金額
          <input
            class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 text-lg outline-none focus:border-[#39705d]"
            type="number"
            min="1"
            step="1"
            placeholder="0"
            required
            bind:value={amount}
          />
        </label>
        <label class="grid gap-1.5 text-sm font-semibold">
          カテゴリ
          <select
            class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]"
            required
            bind:value={categoryId}
          >
            {#each formKind === "expense" ? expenseCategories : incomeCategories as category (category.id)}
              <option value={category.id}>{category.name}</option
              >{/each}</select
          >
        </label>
        {#if formKind === "expense"}
          <label class="grid gap-1.5 text-sm font-semibold">
            支払方法
            <select
              class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]"
              required
              bind:value={paymentMethodId}
            >
              {#each paymentMethods as method (method.id)}
                <option value={method.id}>
                  {method.name}
                </option>
              {/each}
            </select>
          </label>
        {/if}
        <label class="grid gap-1.5 text-sm font-semibold"
          >メモ <span class="font-normal text-[#838b86]">（任意）</span
          ><textarea
            class="min-h-20 resize-y rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]"
            placeholder="例：スーパーで食料品"
            bind:value={description}></textarea></label
        >
        <div class="mt-2 flex justify-end gap-3">
          <button
            class="rounded-xl px-4 py-2.5 text-sm font-semibold hover:bg-[#eae7df]"
            type="button"
            onclick={() => (formKind = null)}>キャンセル</button
          ><button
            class="rounded-xl bg-[#245c4a] px-5 py-2.5 text-sm font-bold text-white disabled:opacity-50"
            type="submit"
            disabled={saving}>{saving ? "登録中…" : "登録する"}</button
          >
        </div>
      </form>
    </div>
  </div>
{/if}

{#if recurringFormOpen}
  <div
    class="fixed inset-0 z-50 grid place-items-center overflow-y-auto bg-[#15241e]/55 p-4"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) recurringFormOpen = false;
    }}
  >
    <div
      class="my-auto w-full max-w-2xl rounded-2xl bg-[#fbfaf6] p-6 shadow-2xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="recurring-form-title"
    >
      <div class="flex items-center justify-between">
        <div><p class="text-xs font-bold tracking-widest text-[#39705d]">FIXED EXPENSE</p><h2 class="mt-1 font-serif text-2xl font-semibold" id="recurring-form-title">固定費を登録</h2></div>
        <button class="grid size-9 place-items-center rounded-full hover:bg-[#eae7df]" aria-label="閉じる" onclick={() => recurringFormOpen = false}>✕</button>
      </div>
      <p class="mt-2 text-sm text-[#717b75]">毎月発生する家賃や通信費などの支出予定を登録します。</p>
      <form class="mt-6 grid gap-4 sm:grid-cols-2" onsubmit={submitRecurringExpense}>
        <label class="grid gap-1.5 text-sm font-semibold sm:col-span-2">名称<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" placeholder="例：家賃" required bind:value={recurringName} /></label>
        <label class="grid gap-1.5 text-sm font-semibold">金額<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" type="number" min="1" step="1" placeholder="0" required bind:value={recurringAmount} /></label>
        <label class="grid gap-1.5 text-sm font-semibold">毎月の支払日<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" type="number" min="1" max="31" required bind:value={recurringPaymentDay} /></label>
        <label class="grid gap-1.5 text-sm font-semibold">カテゴリ<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" required bind:value={recurringCategoryId}>{#each expenseCategories as category (category.id)}<option value={category.id}>{category.name}</option>{/each}</select></label>
        <label class="grid gap-1.5 text-sm font-semibold">支払方法<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" required bind:value={recurringPaymentMethodId}>{#each paymentMethods as method (method.id)}<option value={method.id}>{method.name}</option>{/each}</select></label>
        <label class="grid gap-1.5 text-sm font-semibold">開始日<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" type="date" required bind:value={recurringStartDate} /></label>
        <label class="grid gap-1.5 text-sm font-semibold"><span>終了日 <span class="font-normal text-[#838b86]">（任意）</span></span><input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" type="date" min={recurringStartDate} bind:value={recurringEndDate} /></label>
        <label class="grid gap-1.5 text-sm font-semibold sm:col-span-2"><span>備考 <span class="font-normal text-[#838b86]">（任意）</span></span><textarea class="min-h-20 resize-y rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 outline-none focus:border-[#39705d]" placeholder="契約内容など" bind:value={recurringDescription}></textarea></label>
        <label class="flex items-center gap-2 text-sm font-semibold sm:col-span-2"><input class="size-4 accent-[#245c4a]" type="checkbox" bind:checked={recurringIsActive} />登録後すぐに有効にする</label>
        <div class="mt-2 flex justify-end gap-3 sm:col-span-2"><button class="rounded-xl px-4 py-2.5 text-sm font-semibold hover:bg-[#eae7df]" type="button" onclick={() => recurringFormOpen = false}>キャンセル</button><button class="rounded-xl bg-[#245c4a] px-5 py-2.5 text-sm font-bold text-white disabled:opacity-50" type="submit" disabled={saving}>{saving ? "登録中…" : "固定費を登録"}</button></div>
      </form>
    </div>
  </div>
{/if}
