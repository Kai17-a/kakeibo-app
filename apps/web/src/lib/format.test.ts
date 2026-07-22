import { describe, expect, it } from 'vitest';
import { formatDate, formatYen } from './format';

describe('format helpers', () => {
  it('formats a yen amount', () => expect(formatYen('12800')).toContain('12,800'));
  it('formats an API date in Japanese', () => expect(formatDate('2026-07-19')).toContain('19'));
});
