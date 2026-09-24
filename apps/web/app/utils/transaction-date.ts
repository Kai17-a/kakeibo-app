/**
 * The date suggested for a new transaction in `month` (YYYY-MM): today's day of the month,
 * capped at the 28th so that the date exists in every month.
 */
export function defaultTransactionDate(month: string, today: Date = new Date()): string {
  return `${month}-${String(Math.min(today.getDate(), 28)).padStart(2, "0")}`;
}
