import { currentMonth, currentYear, isValidMonth, isValidYear } from "~/utils/format";

/** The month (YYYY-MM) selected through `?month=` on the page at `path`; defaults to this month. */
export function useMonthQuery(path: string) {
  const route = useRoute();
  const month = computed(() =>
    typeof route.query.month === "string" && isValidMonth(route.query.month)
      ? route.query.month
      : currentMonth(),
  );
  function setMonth(next: string) {
    navigateTo({ path, query: { month: next } }, { replace: true });
  }
  return { month, setMonth };
}

/** The year (YYYY) selected through `?year=` on the page at `path`; defaults to this year. */
export function useYearQuery(path: string) {
  const route = useRoute();
  const year = computed(() =>
    typeof route.query.year === "string" && isValidYear(route.query.year)
      ? route.query.year
      : currentYear(),
  );
  function setYear(next: string) {
    navigateTo({ path, query: { year: next } }, { replace: true });
  }
  return { year, setYear };
}
