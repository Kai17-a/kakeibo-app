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

  it('creates an expense category', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/expense-categories');
      expect(request.method).toBe('POST');
      expect(await request.json()).toEqual({ name: '食費', description: '日々の食事' });
      return Response.json({ id: 'food', name: '食費', description: '日々の食事' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createExpenseCategory({ name: '食費', description: '日々の食事' });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('creates an income category', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/income-categories');
      expect(request.method).toBe('POST');
      expect(await request.json()).toEqual({ name: '給与', description: null });
      return Response.json({ id: 'salary', name: '給与', description: null });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createIncomeCategory({ name: '給与', description: null });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('posts CSV for expense import', async () => {
    const csv = '日付,金額,カテゴリ,支払方法,メモ\n2026-07-22,1200,食費,現金,';
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/import/expenses');
      expect(request.method).toBe('POST');
      expect(request.headers.get('content-type')).toContain('text/csv');
      expect(await request.text()).toBe(csv);
      return Response.json({ imported: 1, created_categories: [], created_payment_methods: [] });
    });
    vi.stubGlobal('fetch', fetchMock);

    const result = await api.importExpenses(csv);

    expect(fetchMock).toHaveBeenCalledOnce();
    expect(result.imported).toBe(1);
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
