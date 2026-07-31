import { describe, expect, it } from 'vitest';
import { currentDate, currentMonth, formatDate, formatYen } from './format';

describe('format helpers', () => {
  it('formats a yen amount', () => expect(formatYen('12800')).toContain('12,800'));
  it('formats an API date in Japanese', () => expect(formatDate('2026-07-19')).toContain('19'));
  it('returns the current month in local time', () => {
    const now = new Date();
    const expected = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
    expect(currentMonth()).toBe(expected);
  });
  it('returns the current date in local time', () => {
    const now = new Date();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    expect(currentDate()).toBe(`${now.getFullYear()}-${month}-${day}`);
  });
});
