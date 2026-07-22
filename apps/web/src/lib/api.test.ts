// @vitest-environment node

import { afterEach, describe, expect, it, vi } from 'vitest';
import { api } from './api';

afterEach(() => vi.unstubAllGlobals());

describe('ky API client', () => {
  it('sends JSON through ky', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.method).toBe('POST');
      expect(request.headers.get('content-type')).toContain('application/json');
      expect(await request.json()).toMatchObject({ amount: '1200', category_id: 'income' });
      return Response.json({ id: '1', amount: '1200' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createIncome({
      transaction_date: '2026-07-01',
      amount: '1200',
      category_id: 'income',
      description: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('converts HTTP errors to ApiError', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => Response.json({ message: '入力が不正です' }, { status: 400 })),
    );

    await expect(api.expenses()).rejects.toEqual(
      expect.objectContaining({ status: 400, message: '入力が不正です' }),
    );
  });

  it('handles empty delete responses', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => new Response(null, { status: 204 })),
    );
    await expect(api.deleteExpense('expense-id')).resolves.toBeUndefined();
  });
});
