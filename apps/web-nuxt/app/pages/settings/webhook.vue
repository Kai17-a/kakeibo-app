<script setup lang="ts">
import type { WebhookUrl, WebhookUrlInput } from '~/types/webhook'
import { validateWebhook, webhookEvents } from '~/utils/webhook'

useSeoMeta({ title: 'Webhook設定' })
const api = useApi()
const toast = useToast()
const items = ref<WebhookUrl[]>([])
const loading = ref(true)
const loadError = ref('')
const formOpen = ref(false)
const deleteOpen = ref(false)
const editingId = ref<string | null>(null)
const deleting = ref<WebhookUrl | null>(null)
const saving = ref(false)
const removing = ref(false)
const formError = ref('')
const deleteError = ref('')
const state = reactive<Omit<WebhookUrlInput, 'description'> & { description: string }>({ url: '', description: '', is_active: true, events: [] })

async function load() {
  loading.value = true
  loadError.value = ''
  try {
    items.value = await api.webhookUrls()
  } catch (error) {
    loadError.value = apiErrorMessage(error)
  } finally {
    loading.value = false
  }
}

// Client-side loading keeps prerendering independent of the Rust server.
onMounted(load)

function openForm(item?: WebhookUrl) {
  editingId.value = item?.id ?? null
  Object.assign(state, {
    url: item?.url ?? '',
    description: item?.description ?? '',
    is_active: item?.is_active ?? true,
    events: item ? [...item.events] : webhookEvents.map(event => event.value)
  })
  formError.value = ''
  formOpen.value = true
}

function toggleEvent(value: string, checked: boolean | 'indeterminate') {
  state.events = state.events.filter(event => event !== value)
  if (checked === true) state.events.push(value)
}

async function save() {
  if (saving.value) return
  saving.value = true
  formError.value = ''
  try {
    const input = { ...state, url: state.url.trim(), description: state.description?.trim() || null, events: [...state.events] }
    const result = editingId.value
      ? await api.updateWebhookUrl(editingId.value, input)
      : await api.createWebhookUrl(input)
    items.value = editingId.value
      ? items.value.map(item => item.id === result.id ? result : item)
      : [...items.value, result]
    formOpen.value = false
    toast.add({ title: editingId.value ? 'Webhook URLを更新しました' : 'Webhook URLを追加しました', color: 'success' })
  } catch (error) {
    formError.value = apiErrorMessage(error)
  } finally {
    saving.value = false
  }
}

function askDelete(item: WebhookUrl) {
  deleting.value = item
  deleteError.value = ''
  deleteOpen.value = true
}

async function remove() {
  if (!deleting.value || removing.value) return
  removing.value = true
  deleteError.value = ''
  try {
    await api.deleteWebhookUrl(deleting.value.id)
    items.value = items.value.filter(item => item.id !== deleting.value?.id)
    deleteOpen.value = false
    toast.add({ title: 'Webhook URLを削除しました', color: 'success' })
  } catch (error) {
    deleteError.value = apiErrorMessage(error)
  } finally {
    removing.value = false
  }
}
</script>

<template>
  <section
    class="space-y-6"
    aria-labelledby="webhook-heading"
  >
    <div class="flex items-start justify-between gap-4">
      <div class="space-y-2">
        <h2
          id="webhook-heading"
          class="text-lg font-semibold flex items-center gap-3"
        >
          Webhook
          <UBadge
            v-if="!loading && !loadError"
            color="neutral"
            variant="soft"
          >
            {{ items.length }}件
          </UBadge>
        </h2>
        <p class="text-sm text-muted">
          支出・収入の登録時や予算超過時に通知を送信するWebhook URLを管理します。
        </p>
      </div>
      <UButton
        icon="i-lucide-plus"
        class="shrink-0"
        :disabled="loading || !!loadError"
        @click="openForm()"
      >
        追加
      </UButton>
    </div>

    <div
      v-if="loading"
      role="status"
      aria-label="Webhook URLを読み込み中"
      class="space-y-3"
    >
      <USkeleton
        v-for="i in 3"
        :key="i"
        class="h-32 w-full"
      />
      <span class="sr-only">読み込み中…</span>
    </div>
    <UAlert
      v-else-if="loadError"
      color="error"
      title="Webhook URLを取得できませんでした"
      :description="loadError"
      :actions="[{ label: '再試行', color: 'error', variant: 'outline', onClick: load }]"
    />
    <UCard
      v-else-if="!items.length"
      class="text-center"
    >
      <div class="py-10 space-y-3">
        <UIcon
          name="i-lucide-webhook"
          class="size-8 text-muted"
        />
        <h2 class="font-semibold">
          Webhook URLがありません
        </h2>
        <p class="text-sm text-muted">
          右上の「追加」から通知を送信するURLを登録してください。
        </p>
      </div>
    </UCard>
    <ul
      v-else
      class="space-y-4"
      aria-label="登録済みのWebhook URL一覧"
    >
      <li
        v-for="item in items"
        :key="item.id"
      >
        <UCard>
          <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
            <div class="min-w-0 space-y-3">
              <p class="font-medium break-all">
                {{ item.url }}
              </p>
              <p class="text-sm text-muted whitespace-pre-wrap break-words">
                {{ item.description || '説明はありません' }}
              </p>
              <div class="flex flex-wrap gap-2">
                <UBadge
                  color="neutral"
                  :variant="item.is_active ? 'outline' : 'soft'"
                >
                  {{ item.is_active ? '有効' : '無効' }}
                </UBadge>
                <UBadge
                  v-for="event in item.events"
                  :key="event"
                  color="neutral"
                  variant="soft"
                >
                  {{ webhookEvents.find(option => option.value === event)?.label ?? event }}
                </UBadge>
              </div>
            </div>
            <div class="flex gap-2 shrink-0 self-end sm:self-start">
              <UButton
                color="neutral"
                variant="ghost"
                :aria-label="`${item.url}を編集`"
                @click="openForm(item)"
              >
                編集
              </UButton>
              <UButton
                color="error"
                variant="soft"
                :aria-label="`${item.url}を削除`"
                @click="askDelete(item)"
              >
                削除
              </UButton>
            </div>
          </div>
        </UCard>
      </li>
    </ul>

    <UModal
      v-model:open="formOpen"
      :title="editingId ? 'Webhook URLを編集' : 'Webhook URLを追加'"
      description="支出・収入の登録時や予算超過時に通知を送信するURLを設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="webhook-form"
          :state="state"
          :validate="validateWebhook"
          :disabled="saving"
          class="space-y-5"
          @submit="save"
        >
          <UAlert
            v-if="formError"
            color="error"
            title="保存できませんでした"
            :description="formError"
          />
          <UFormField
            name="url"
            label="Webhook URL"
            required
          >
            <UInput
              v-model="state.url"
              type="url"
              placeholder="https://example.com/webhook"
              class="w-full"
              autofocus
            />
          </UFormField>
          <UFormField
            name="description"
            label="説明（任意）"
          >
            <UTextarea
              v-model="state.description"
              :maxlength="500"
              placeholder="通知先の用途やメモを入力"
              class="w-full"
            />
          </UFormField>
          <UFormField name="is_active">
            <USwitch
              v-model="state.is_active"
              label="通知を有効にする"
            />
          </UFormField>
          <UFormField
            name="events"
            required
          >
            <fieldset class="space-y-3">
              <legend class="text-sm font-medium mb-3">
                通知するイベント
              </legend>
              <UCheckbox
                v-for="event in webhookEvents"
                :key="event.value"
                :model-value="state.events.includes(event.value)"
                :label="event.label"
                @update:model-value="toggleEvent(event.value, $event)"
              />
            </fieldset>
          </UFormField>
        </UForm>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          :disabled="saving"
          @click="formOpen = false"
        >
          キャンセル
        </UButton>
        <UButton
          type="submit"
          form="webhook-form"
          :loading="saving"
          :disabled="saving"
        >
          {{ editingId ? '更新' : '追加' }}
        </UButton>
      </template>
    </UModal>

    <UModal
      v-model:open="deleteOpen"
      title="Webhook URLを削除"
      description="この通知先を削除します。この操作は取り消せません。"
      :dismissible="!removing"
      :close="!removing"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <p class="break-all">
          {{ deleting?.url }}
        </p>
        <UAlert
          v-if="deleteError"
          color="error"
          title="削除できませんでした"
          :description="deleteError"
          class="mt-4"
        />
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          :disabled="removing"
          @click="deleteOpen = false"
        >
          キャンセル
        </UButton>
        <UButton
          color="error"
          :loading="removing"
          :disabled="removing"
          @click="remove"
        >
          削除
        </UButton>
      </template>
    </UModal>
  </section>
</template>
