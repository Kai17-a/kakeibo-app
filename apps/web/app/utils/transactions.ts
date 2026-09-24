import type { Transaction } from "./summaries";

export interface TransactionDayGroup<T extends Transaction = Transaction> {
  /** The shared transaction date (YYYY-MM-DD). */
  date: string;
  items: T[];
  /** Incomes minus expenses on that date. */
  total: number;
}

/** Groups transactions by date, keeping the input order of dates and of items within a date. */
export function groupTransactionsByDate<T extends Transaction>(
  transactions: T[],
): TransactionDayGroup<T>[] {
  const groups = new Map<string, T[]>();
  for (const item of transactions) {
    const group = groups.get(item.transaction_date) ?? [];
    group.push(item);
    groups.set(item.transaction_date, group);
  }
  return [...groups].map(([date, items]) => ({
    date,
    items,
    total: items.reduce(
      (sum, item) => sum + (item.kind === "income" ? Number(item.amount) : -Number(item.amount)),
      0,
    ),
  }));
}
