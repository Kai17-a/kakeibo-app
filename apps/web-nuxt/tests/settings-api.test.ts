import assert from 'node:assert/strict'
import { test } from 'node:test'
import { useSettingsApi } from '../app/composables/useSettingsApi.ts'

test('settings API uses the correct endpoints, queries and mutation payloads without retries', async () => {
  const calls: { path: string, options: Record<string, unknown> }[] = []
  const response = { items: [{ id: 'category' }] }
  const client = async (path: string, options: Record<string, unknown> = {}) => {
    calls.push({ path, options })
    return response
  }
  const original = Object.getOwnPropertyDescriptor(globalThis, '$fetch')
  Object.defineProperty(globalThis, '$fetch', {
    configurable: true,
    value: {
      create(options: Record<string, unknown>) {
        assert.deepEqual(options, { retry: 0, timeout: 15000 })
        return client
      }
    }
  })
  try {
    const api = useSettingsApi()
    for (const kind of ['expense', 'income'] as const) {
      const categories = api.categories(kind)
      assert.deepEqual(await categories.list(), response.items)
      assert.deepEqual(calls.at(-1), {
        path: `/api/${kind}-categories`,
        options: { query: { sort_by: 'display_order', sort_order: 'asc', per_page: 100 } }
      })
      const input = { name: 'カテゴリ', description: null, parent_category_id: null }
      await categories.create(input)
      assert.deepEqual(calls.at(-1), { path: `/api/${kind}-categories`, options: { method: 'POST', body: input } })
      await categories.update('a/b', input)
      assert.deepEqual(calls.at(-1), { path: `/api/${kind}-categories/a%2Fb`, options: { method: 'PUT', body: input } })
      const order = { parent_category_id: null, category_ids: ['b', 'a'] }
      await categories.reorder(order)
      assert.deepEqual(calls.at(-1), { path: `/api/${kind}-categories/order`, options: { method: 'PUT', body: order } })
      await categories.remove('a/b')
      assert.deepEqual(calls.at(-1), { path: `/api/${kind}-categories/a%2Fb`, options: { method: 'DELETE' } })
    }
    assert.deepEqual(await api.paymentMethods.list(), response.items)
    assert.deepEqual(calls.at(-1), { path: '/api/payment-methods', options: { query: { sort_by: 'name', sort_order: 'asc', per_page: 100 } } })
    assert.equal(await api.budgets.list(), response)
    assert.deepEqual(calls.at(-1), { path: '/api/budgets', options: {} })
    for (const [resource, path, input] of [
      [api.paymentMethods, '/api/payment-methods', { name: '現金', description: null, initial_balance: '0' }],
      [api.budgets, '/api/budgets', { category_id: 'a', amount: '1000' }]
    ] as const) {
      // Each tuple pairs its resource with that resource's input shape.
      if ('name' in input) {
        await api.paymentMethods.create(input)
        await api.paymentMethods.update('id', input)
      } else {
        await api.budgets.create(input)
        await api.budgets.update('id', input)
      }
      assert.deepEqual(calls.at(-2), { path, options: { method: 'POST', body: input } })
      assert.deepEqual(calls.at(-1), { path: `${path}/id`, options: { method: 'PUT', body: input } })
      await resource.remove('id')
      assert.deepEqual(calls.at(-1), { path: `${path}/id`, options: { method: 'DELETE' } })
    }
  } finally {
    if (original) Object.defineProperty(globalThis, '$fetch', original)
    else Reflect.deleteProperty(globalThis, '$fetch')
  }
})
