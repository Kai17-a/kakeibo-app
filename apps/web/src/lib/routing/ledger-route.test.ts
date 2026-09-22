import { describe, expect, it, vi } from 'vitest';
import { ledgerUrl, parseLedgerRoute } from './ledger-route';

describe('ledger route', () => {
  it('derives the view from the path and parses the period query', () => {
    expect(parseLedgerRoute(new URL('https://example.test/daily?month=2026-07'))).toEqual({
      view: 'daily',
      month: '2026-07',
      year: '2026',
    });
  });

  it('falls back safely for invalid query parameters', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-09-22T12:00:00+09:00'));
    expect(
      parseLedgerRoute(new URL('https://example.test/unknown?month=2026-13&year=nope')),
    ).toEqual({ view: 'monthly', month: '2026-09', year: '2026' });
    vi.useRealTimers();
  });

  it('builds a canonical URL and removes irrelevant period parameters', () => {
    const current = new URL('https://example.test/daily?month=2026-07&year=2026');
    expect(ledgerUrl({ view: 'annual', month: '2026-07', year: '2025' }, current)).toBe(
      '/annual?year=2025',
    );
  });
});
