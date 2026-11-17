export const settingsNavigation = [
  {
    label: "予算",
    icon: "i-lucide-piggy-bank",
    description: "カテゴリごとの月額予算",
    pages: [{ label: "予算", to: "/settings/budget" }],
  },
  {
    label: "定期の収支",
    icon: "i-lucide-repeat",
    description: "毎月の支出・収入・振替予定",
    pages: [
      { label: "支出", to: "/settings/recurring-expenses" },
      { label: "収入", to: "/settings/recurring-incomes" },
      { label: "振替", to: "/settings/recurring-transfers" },
    ],
  },
  {
    label: "カテゴリと支払方法",
    icon: "i-lucide-tags",
    description: "収支の分類と支払方法・残高",
    pages: [
      { label: "支出カテゴリ", to: "/settings/expense-categories" },
      { label: "収入カテゴリ", to: "/settings/income-categories" },
      { label: "支払方法", to: "/settings/payment-methods" },
    ],
  },
  {
    label: "データ管理",
    icon: "i-lucide-database",
    description: "データの取り込みとバックアップ",
    pages: [{ label: "データ管理", to: "/settings/data" }],
  },
];

export function findSettingsGroup(path: string) {
  return settingsNavigation.find((group) =>
    group.pages.some((page) => page.to === path.replace(/\/$/, "")),
  );
}
