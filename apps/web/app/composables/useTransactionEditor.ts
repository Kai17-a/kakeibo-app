import type { RecurringExpense, RecurringIncome } from "~/types/settings";
import type {
  ExchangeRatePreview,
  Expense,
  Income,
  TransactionSubmission,
} from "~/types/transactions";
import { defaultTransactionDate } from "~/utils/transaction-date";

interface TransactionEditorOptions {
  /** The month (YYYY-MM) new transactions default to. */
  month: MaybeRefOrGetter<string>;
  /** The page-owned lists, updated in place after a successful mutation. */
  expenses: Ref<Expense[]>;
  incomes: Ref<Income[]>;
}

type DeleteTarget = { kind: "expense"; item: Expense } | { kind: "income"; item: Income };

/**
 * State and actions for the transaction form modal (create, edit, register a variable recurring
 * item) and for the transaction delete confirmation.
 */
export function useTransactionEditor({ month, expenses, incomes }: TransactionEditorOptions) {
  const transactionsApi = useTransactionsApi();
  const toast = useToast();

  // --- form modal ---
  const formOpen = ref(false);
  const editingExpense = ref<Expense | null>(null);
  const editingIncome = ref<Income | null>(null);
  const recurringExpensePreset = ref<RecurringExpense | null>(null);
  const recurringIncomePreset = ref<RecurringIncome | null>(null);
  const exchangePreview = ref<ExchangeRatePreview | null>(null);
  const saving = ref(false);
  const initialDate = computed(() => defaultTransactionDate(toValue(month)));

  function resetForm() {
    editingExpense.value = null;
    editingIncome.value = null;
    recurringExpensePreset.value = null;
    recurringIncomePreset.value = null;
    exchangePreview.value = null;
  }

  function openNew() {
    resetForm();
    formOpen.value = true;
  }

  function editExpense(item: Expense) {
    resetForm();
    editingExpense.value = item;
    formOpen.value = true;
  }

  function editIncome(item: Income) {
    resetForm();
    editingIncome.value = item;
    formOpen.value = true;
  }

  /** Opens the form prefilled from a variable recurring expense, converting USD amounts first. */
  async function registerRecurringExpense(item: RecurringExpense) {
    resetForm();
    recurringExpensePreset.value = item;
    if (item.currency_code === "USD") {
      try {
        exchangePreview.value = await transactionsApi.previewRecurringExpenseExchangeRate(
          item.id,
          toValue(month),
        );
      } catch (error) {
        toast.add({
          title: apiErrorMessage(error),
          description: "為替レートを取得できませんでした。金額を手動で入力してください。",
          color: "warning",
        });
      }
    }
    formOpen.value = true;
  }

  function registerRecurringIncome(item: RecurringIncome) {
    resetForm();
    recurringIncomePreset.value = item;
    formOpen.value = true;
  }

  function closeForm() {
    formOpen.value = false;
    resetForm();
  }

  async function persist(submission: TransactionSubmission): Promise<string> {
    if (submission.kind === "expense") {
      const editing = editingExpense.value;
      if (editing) {
        const updated = await transactionsApi.updateExpense(editing.id, submission.input);
        expenses.value = expenses.value.map((item) => (item.id === updated.id ? updated : item));
        return "支出を更新しました";
      }
      expenses.value = [await transactionsApi.createExpense(submission.input), ...expenses.value];
      return "支出を登録しました";
    }
    const editing = editingIncome.value;
    if (editing) {
      const updated = await transactionsApi.updateIncome(editing.id, submission.input);
      incomes.value = incomes.value.map((item) => (item.id === updated.id ? updated : item));
      return "収入を更新しました";
    }
    incomes.value = [await transactionsApi.createIncome(submission.input), ...incomes.value];
    return "収入を登録しました";
  }

  /** Saves the submission; returns whether it succeeded so the form can reset for the next entry. */
  async function save(submission: TransactionSubmission, keepOpen: boolean): Promise<boolean> {
    saving.value = true;
    try {
      toast.add({ title: await persist(submission), color: "success" });
      if (!keepOpen) closeForm();
      return true;
    } catch (error) {
      toast.add({ title: apiErrorMessage(error), color: "error" });
      return false;
    } finally {
      saving.value = false;
    }
  }

  // --- delete confirmation ---
  const deleteTarget = ref<DeleteTarget | null>(null);
  const deleteOpen = computed({
    get: () => deleteTarget.value !== null,
    set: (value: boolean) => {
      if (!value) deleteTarget.value = null;
    },
  });
  const deleteDescription = computed(() =>
    deleteTarget.value
      ? `この${deleteTarget.value.kind === "expense" ? "支出" : "収入"}明細を削除しますか？`
      : "",
  );
  const removing = ref(false);

  function askDeleteExpense(item: Expense) {
    deleteTarget.value = { kind: "expense", item };
  }

  function askDeleteIncome(item: Income) {
    deleteTarget.value = { kind: "income", item };
  }

  async function confirmDelete() {
    const target = deleteTarget.value;
    if (!target) return;
    removing.value = true;
    try {
      if (target.kind === "expense") {
        await transactionsApi.deleteExpense(target.item.id);
        expenses.value = expenses.value.filter((item) => item.id !== target.item.id);
      } else {
        await transactionsApi.deleteIncome(target.item.id);
        incomes.value = incomes.value.filter((item) => item.id !== target.item.id);
      }
      toast.add({ title: "明細を削除しました", color: "success" });
      deleteTarget.value = null;
    } catch (error) {
      toast.add({ title: apiErrorMessage(error), color: "error" });
    } finally {
      removing.value = false;
    }
  }

  return {
    formOpen,
    editingExpense,
    editingIncome,
    recurringExpensePreset,
    recurringIncomePreset,
    exchangePreview,
    saving,
    initialDate,
    openNew,
    editExpense,
    editIncome,
    registerRecurringExpense,
    registerRecurringIncome,
    closeForm,
    save,
    deleteOpen,
    deleteDescription,
    removing,
    askDeleteExpense,
    askDeleteIncome,
    confirmDelete,
  };
}
