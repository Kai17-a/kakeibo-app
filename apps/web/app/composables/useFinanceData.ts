import type {
  Budget,
  Category,
  PaymentMethod,
  RecurringExpense,
  RecurringIncome,
} from "~/types/settings";
import type { Expense, Income } from "~/types/transactions";

interface FinanceDataOptions {
  /** Also load income categories (needed to register or edit incomes). */
  incomeCategories?: boolean;
  /** Also load recurring incomes. */
  recurringIncomes?: boolean;
  /** Also load budgets. */
  budgets?: boolean;
}

/**
 * Loads the transactions and settings a summary page needs, in parallel, once the page is mounted.
 * Expenses, incomes, expense categories, payment methods and recurring expenses are always loaded;
 * the rest only when requested. `load` can be called again to retry after `loadError`.
 */
export function useFinanceData(options: FinanceDataOptions = {}) {
  const settingsApi = useSettingsApi();
  const transactionsApi = useTransactionsApi();

  const expenses = ref<Expense[]>([]);
  const incomes = ref<Income[]>([]);
  const expenseCategories = ref<Category[]>([]);
  const incomeCategories = ref<Category[]>([]);
  const paymentMethods = ref<PaymentMethod[]>([]);
  const recurringExpenses = ref<RecurringExpense[]>([]);
  const recurringIncomes = ref<RecurringIncome[]>([]);
  const budgets = ref<Budget[]>([]);
  const loading = ref(true);
  const loadError = ref("");

  async function load() {
    loading.value = true;
    loadError.value = "";
    try {
      const [
        expenseData,
        incomeData,
        expenseCategoryData,
        incomeCategoryData,
        paymentData,
        recurringExpenseData,
        recurringIncomeData,
        budgetData,
      ] = await Promise.all([
        transactionsApi.expenses(),
        transactionsApi.incomes(),
        settingsApi.categories("expense").list(),
        options.incomeCategories ? settingsApi.categories("income").list() : [],
        settingsApi.paymentMethods.list(),
        settingsApi.recurringExpenses.list(),
        options.recurringIncomes ? settingsApi.recurringIncomes.list() : [],
        options.budgets ? settingsApi.budgets.list() : [],
      ]);
      expenses.value = expenseData;
      incomes.value = incomeData;
      expenseCategories.value = expenseCategoryData;
      incomeCategories.value = incomeCategoryData;
      paymentMethods.value = paymentData;
      recurringExpenses.value = recurringExpenseData;
      recurringIncomes.value = recurringIncomeData;
      budgets.value = budgetData;
    } catch (error) {
      loadError.value = apiErrorMessage(error);
    } finally {
      loading.value = false;
    }
  }

  onMounted(load);

  return {
    expenses,
    incomes,
    expenseCategories,
    incomeCategories,
    paymentMethods,
    recurringExpenses,
    recurringIncomes,
    budgets,
    loading,
    loadError,
    load,
  };
}
