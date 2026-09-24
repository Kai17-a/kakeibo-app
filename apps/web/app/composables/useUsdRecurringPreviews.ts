import type { RecurringExpense } from "~/types/settings";
import type { ExchangeRatePreview } from "~/types/transactions";

/**
 * Fetches the JPY conversion of each active fixed USD recurring expense for the given months,
 * refetching when the months or the recurring expenses change. Failed conversions are left out.
 *
 * @returns the previews keyed by month (YYYY-MM) and then by recurring expense id
 */
export function useUsdRecurringPreviews(
  months: MaybeRefOrGetter<string[]>,
  recurringExpenses: Ref<RecurringExpense[]>,
) {
  const transactionsApi = useTransactionsApi();
  const previews = ref(new Map<string, Map<string, ExchangeRatePreview>>());
  // Only the latest request may write the result, so a slow earlier response can't win.
  let latestRequestId = 0;

  watch(
    [() => toValue(months).join(), recurringExpenses],
    async () => {
      const requestId = ++latestRequestId;
      const targets = recurringExpenses.value.filter(
        (item) => item.is_active && !item.is_variable && item.currency_code === "USD",
      );
      if (!targets.length) {
        previews.value = new Map();
        return;
      }
      const results = await Promise.allSettled(
        toValue(months).flatMap((month) =>
          targets.map(
            async (item) =>
              [
                month,
                item.id,
                await transactionsApi.previewRecurringExpenseExchangeRate(item.id, month),
              ] as const,
          ),
        ),
      );
      if (requestId !== latestRequestId) return;
      const grouped = new Map<string, Map<string, ExchangeRatePreview>>();
      for (const result of results) {
        if (result.status !== "fulfilled") continue;
        const [month, id, preview] = result.value;
        if (!grouped.has(month)) grouped.set(month, new Map());
        grouped.get(month)!.set(id, preview);
      }
      previews.value = grouped;
    },
    { immediate: true },
  );

  return previews;
}
