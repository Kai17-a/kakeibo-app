import type {
  BackfillResponse,
  Budget,
  BudgetInput,
  Category,
  CategoryInput,
  CategoryKind,
  CategoryReorderInput,
  PaymentMethod,
  PaymentMethodInput,
  PendingMonthsResponse,
  RecurringExpense,
  RecurringExpenseInput,
  RecurringIncome,
  RecurringIncomeInput,
} from "~/types/settings";

export function useSettingsApi() {
  const client = $fetch.create({ retry: 0, timeout: 15000 });
  function resource<T, Input>(path: string) {
    return {
      create: (body: Input) =>
        client<T>(path, { method: "POST", body: body as Record<string, unknown> }),
      update: (id: string, body: Input) =>
        client<T>(`${path}/${encodeURIComponent(id)}`, {
          method: "PUT",
          body: body as Record<string, unknown>,
        }),
      remove: async (id: string): Promise<void> => {
        await client(`${path}/${encodeURIComponent(id)}`, { method: "DELETE" });
      },
    };
  }
  return {
    categories: (kind: CategoryKind) => {
      const path = `/api/${kind}-categories`;
      return {
        ...resource<Category, CategoryInput>(path),
        list: async () =>
          (
            await client<{ items: Category[] }>(path, {
              query: { sort_by: "display_order", sort_order: "asc", per_page: 100 },
            })
          ).items,
        reorder: async (body: CategoryReorderInput): Promise<void> => {
          await client(`${path}/order`, { method: "PUT", body });
        },
      };
    },
    paymentMethods: {
      ...resource<PaymentMethod, PaymentMethodInput>("/api/payment-methods"),
      list: async () =>
        (
          await client<{ items: PaymentMethod[] }>("/api/payment-methods", {
            query: { sort_by: "name", sort_order: "asc", per_page: 100 },
          })
        ).items,
    },
    recurringIncomes: {
      ...resource<RecurringIncome, RecurringIncomeInput>("/api/recurring-incomes"),
      list: () => client<RecurringIncome[]>("/api/recurring-incomes"),
    },
    recurringExpenses: {
      ...resource<RecurringExpense, RecurringExpenseInput>("/api/recurring-expenses"),
      list: () => client<RecurringExpense[]>("/api/recurring-expenses"),
      pendingMonths: (id: string) =>
        client<PendingMonthsResponse>(
          `/api/recurring-expenses/${encodeURIComponent(id)}/pending-months`,
        ),
      backfill: (id: string) =>
        client<BackfillResponse>(`/api/recurring-expenses/${encodeURIComponent(id)}/backfill`, {
          method: "POST",
        }),
    },
    budgets: {
      ...resource<Budget, BudgetInput>("/api/budgets"),
      list: () => client<Budget[]>("/api/budgets"),
    },
  };
}
