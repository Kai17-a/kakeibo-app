import { currentMonth } from '$lib/format';

export const ledgerViews = ['monthly', 'daily', 'annual'] as const;
export type LedgerView = (typeof ledgerViews)[number];

export interface LedgerRouteState {
  view: LedgerView;
  month: string;
  year: string;
}

const monthPattern = /^(\d{4})-(\d{2})$/;
const yearPattern = /^\d{4}$/;

function isValidMonth(value: string | null): value is string {
  if (!value || !monthPattern.test(value)) return false;
  const month = Number(value.slice(5, 7));
  return month >= 1 && month <= 12;
}

function isValidYear(value: string | null): value is string {
  return value !== null && yearPattern.test(value);
}

export function parseLedgerRoute(url: URL): LedgerRouteState {
  const defaultMonth = currentMonth();
  const pathView = url.pathname.replace(/\/+$/, '').slice(1);
  const view = ledgerViews.includes(pathView as LedgerView) ? (pathView as LedgerView) : 'monthly';
  const month = isValidMonth(url.searchParams.get('month'))
    ? url.searchParams.get('month')!
    : defaultMonth;
  const year = isValidYear(url.searchParams.get('year'))
    ? url.searchParams.get('year')!
    : defaultMonth.slice(0, 4);

  return { view, month, year };
}

export function ledgerUrl(state: LedgerRouteState, currentUrl: URL): string {
  const url = new URL(currentUrl);
  url.pathname = `/${state.view}`;
  url.search = '';
  if (state.view === 'annual') {
    url.searchParams.set('year', state.year);
  } else {
    url.searchParams.set('month', state.month);
  }
  return `${url.pathname}${url.search}`;
}
