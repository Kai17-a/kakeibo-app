import type { NavigationMenuItem } from "@nuxt/ui";
import { isValidMonth, isValidYear } from "~/utils/format";

export function useAppNavigation() {
  const route = useRoute();
  return computed(() => {
    const month =
      typeof route.query.month === "string" && isValidMonth(route.query.month)
        ? route.query.month
        : undefined;
    const year =
      typeof route.query.year === "string" && isValidYear(route.query.year)
        ? route.query.year
        : undefined;
    const selectedMonth = month ?? (year ? `${year}-01` : undefined);
    const selectedYear = month?.slice(0, 4) ?? year;

    return [
      [
        {
          label: "月間集計",
          shortLabel: "月間",
          active: route.path === "/monthly",
          icon: "i-lucide-house",
          to: selectedMonth ? { path: "/monthly", query: { month: selectedMonth } } : "/monthly",
          exact: true,
        },
        {
          label: "明細",
          shortLabel: "明細",
          active: route.path === "/daily",
          icon: "i-lucide-calendar-days",
          to: selectedMonth ? { path: "/daily", query: { month: selectedMonth } } : "/daily",
          exact: true,
        },
        {
          label: "年間集計",
          shortLabel: "年間",
          active: route.path === "/",
          icon: "i-lucide-chart-column",
          to: selectedYear ? { path: "/", query: { year: selectedYear } } : "/",
          exact: true,
        },
        {
          label: "設定",
          shortLabel: "設定",
          icon: "i-lucide-settings",
          to: "/settings",
          active: route.path.startsWith("/settings"),
        },
      ],
    ] satisfies (NavigationMenuItem & { shortLabel: string })[][];
  });
}
