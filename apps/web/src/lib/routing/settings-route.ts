export const settingsTabs = [
  'expense',
  'income',
  'payment',
  'budget',
  'recurring',
  'recurring-income',
  'webhook',
] as const;
export type SettingsTab = (typeof settingsTabs)[number];

export function parseSettingsTab(url: URL): SettingsTab {
  const value = url.searchParams.get('tab');
  return settingsTabs.includes(value as SettingsTab) ? (value as SettingsTab) : 'expense';
}

export function settingsUrl(tab: SettingsTab, currentUrl: URL): string {
  const url = new URL(currentUrl);
  url.search = '';
  url.searchParams.set('tab', tab);
  return `${url.pathname}${url.search}`;
}
