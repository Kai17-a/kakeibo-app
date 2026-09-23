import { describe, expect, it } from 'vitest';
import { parseSettingsTab, settingsUrl } from './settings-route';

describe('settings route', () => {
  it('uses the requested tab and defaults invalid values', () => {
    expect(parseSettingsTab(new URL('https://example.test/settings?tab=budget'))).toBe('budget');
    expect(parseSettingsTab(new URL('https://example.test/settings?tab=invalid'))).toBe('expense');
  });

  it('builds a canonical tab URL', () => {
    expect(settingsUrl('webhook', new URL('https://example.test/settings?tab=expense'))).toBe(
      '/settings?tab=webhook',
    );
  });
});
