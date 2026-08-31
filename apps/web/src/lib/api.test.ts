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
      expect(await request.json()).toEqual({
        name: '食費',
        description: '日々の食事',
        parent_category_id: null,
      });
      return Response.json({ id: 'food', name: '食費', description: '日々の食事' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createExpenseCategory({
      name: '食費',
      description: '日々の食事',
      parent_category_id: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('creates an income category', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/income-categories');
      expect(request.method).toBe('POST');
      expect(await request.json()).toEqual({
        name: '給与',
        description: null,
        parent_category_id: null,
      });
      return Response.json({ id: 'salary', name: '給与', description: null });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createIncomeCategory({ name: '給与', description: null, parent_category_id: null });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('parses and serializes a non-null parent category id', async () => {
    const fetchMock = vi
      .fn()
      .mockImplementationOnce(async () =>
        Response.json({
          items: [
            {
              id: 'dining',
              name: '外食',
              description: null,
              parent_category_id: 'food',
            },
          ],
          pagination: { page: 1, per_page: 100 },
        }),
      )
      .mockImplementationOnce(async (request: Request) => {
        expect(request.method).toBe('POST');
        expect(await request.json()).toEqual({
          name: '外食',
          description: null,
          parent_category_id: 'food',
        });
        return Response.json({
          id: 'dining',
          name: '外食',
          description: null,
          parent_category_id: 'food',
        });
      });
    vi.stubGlobal('fetch', fetchMock);

    const categories = await api.expenseCategories();
    expect(categories.items[0].parent_category_id).toBe('food');
    const created = await api.createExpenseCategory({
      name: '外食',
      description: null,
      parent_category_id: 'food',
    });
    expect(created.parent_category_id).toBe('food');
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('creates a payment method', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/payment-methods');
      expect(request.method).toBe('POST');
      expect(await request.json()).toEqual({ name: 'VISA', description: null });
      return Response.json({ id: 'visa', name: 'VISA', description: null });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createPaymentMethod({ name: 'VISA', description: null });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates a payment method', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/payment-methods/visa');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toEqual({ name: 'VISAカード', description: null });
      return Response.json({ id: 'visa', name: 'VISAカード', description: null });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updatePaymentMethod('visa', { name: 'VISAカード', description: null });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('deletes a payment method', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/payment-methods/visa');
      expect(request.method).toBe('DELETE');
      return new Response(null, { status: 204 });
    });
    vi.stubGlobal('fetch', fetchMock);

    await expect(api.deletePaymentMethod('visa')).resolves.toBeUndefined();
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates an income', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/incomes/income-1');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toEqual({
        transaction_date: '2026-07-01',
        amount: '300000',
        category_id: 'salary',
        description: null,
      });
      return Response.json({ id: 'income-1', amount: '300000' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updateIncome('income-1', {
      transaction_date: '2026-07-01',
      amount: '300000',
      category_id: 'salary',
      description: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates an expense category', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/expense-categories/food');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toEqual({
        name: '食費',
        description: '外食含む',
        parent_category_id: null,
      });
      return Response.json({ id: 'food', name: '食費', description: '外食含む' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updateExpenseCategory('food', {
      name: '食費',
      description: '外食含む',
      parent_category_id: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates an income category', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/income-categories/salary');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toEqual({
        name: '給与',
        description: null,
        parent_category_id: null,
      });
      return Response.json({ id: 'salary', name: '給与', description: null });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updateIncomeCategory('salary', {
      name: '給与',
      description: null,
      parent_category_id: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates a recurring expense', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/recurring-expenses/rec-1');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toMatchObject({ name: '家賃', amount: '100000' });
      return Response.json({ id: 'rec-1', name: '家賃', amount: '100000' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updateRecurringExpense('rec-1', {
      name: '家賃',
      amount: '100000',
      payment_day: 27,
      start_date: '2026-01-01',
      end_date: null,
      category_id: 'housing',
      payment_method_id: 'bank',
      is_active: true,
      is_variable: false,
      description: null,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('creates a webhook url', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/webhook-urls');
      expect(request.method).toBe('POST');
      expect(await request.json()).toEqual({
        url: 'https://example.com/hook',
        description: null,
        is_active: true,
      });
      return Response.json({ id: 'wu-1', url: 'https://example.com/hook' });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.createWebhookUrl({
      url: 'https://example.com/hook',
      description: null,
      is_active: true,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('updates a webhook url', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/webhook-urls/wu-1');
      expect(request.method).toBe('PUT');
      expect(await request.json()).toMatchObject({ is_active: false });
      return Response.json({ id: 'wu-1', is_active: false });
    });
    vi.stubGlobal('fetch', fetchMock);

    await api.updateWebhookUrl('wu-1', {
      url: 'https://example.com/hook',
      description: null,
      is_active: false,
    });

    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('deletes a webhook url', async () => {
    const fetchMock = vi.fn(async (request: Request) => {
      expect(request.url).toBe('http://localhost/api/webhook-urls/wu-1');
      expect(request.method).toBe('DELETE');
      return new Response(null, { status: 204 });
    });
    vi.stubGlobal('fetch', fetchMock);

    await expect(api.deleteWebhookUrl('wu-1')).resolves.toBeUndefined();
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
