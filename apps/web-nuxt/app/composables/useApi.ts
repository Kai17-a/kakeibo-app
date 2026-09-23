import type { WebhookUrl, WebhookUrlInput } from '~/types/webhook'

export function apiErrorMessage(error: unknown): string {
  if (error && typeof error === 'object' && 'data' in error) {
    const data = error.data
    if (data && typeof data === 'object' && 'message' in data && typeof data.message === 'string') {
      return data.message
    }
  }
  return '通信に失敗しました。接続を確認して、もう一度お試しください。'
}

export function useApi() {
  // Mutations must never be retried automatically (especially creation).
  const client = $fetch.create({ retry: 0, timeout: 15000 })
  const path = '/api/webhook-urls'
  return {
    webhookUrls: () => client<WebhookUrl[]>(path),
    createWebhookUrl: (body: WebhookUrlInput) => client<WebhookUrl>(path, { method: 'POST', body }),
    updateWebhookUrl: (id: string, body: WebhookUrlInput) => client<WebhookUrl>(`${path}/${encodeURIComponent(id)}`, { method: 'PUT', body }),
    deleteWebhookUrl: async (id: string): Promise<void> => {
      await client(`${path}/${encodeURIComponent(id)}`, { method: 'DELETE' })
    }
  }
}
