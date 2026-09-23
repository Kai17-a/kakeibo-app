import type { NavigationMenuItem } from '@nuxt/ui'

export const settingsNavigation = [
  { label: '支出カテゴリ', to: '/settings/expense-categories' },
  { label: '収入カテゴリ', to: '/settings/income-categories' },
  { label: '支払方法', to: '/settings/payment-methods' },
  { label: '予算', to: '/settings/budget' },
  { label: '定期支出', to: '/settings/recurring-expenses' },
  { label: '定期収入', to: '/settings/recurring-incomes' },
  { label: 'Webhook', to: '/settings/webhook' },
  { label: 'データ管理', to: '/settings/data' }
] satisfies NavigationMenuItem[]
