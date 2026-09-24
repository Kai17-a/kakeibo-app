<script setup lang="ts">
import type { WebhookUrl, WebhookUrlInput } from "~/types/webhook";
import { validateWebhook, webhookEvents } from "~/utils/webhook";

useSeoMeta({ title: "Webhook設定" });
const label = "Webhook URL";
const api = useApi();
const state = reactive<Omit<WebhookUrlInput, "description"> & { description: string }>({
  url: "",
  description: "",
  is_active: true,
  events: [],
});

// Client-side loading keeps prerendering independent of the Rust server.
const {
  items,
  loading,
  loadError,
  load,
  formOpen,
  editingId,
  saving,
  formError,
  openForm: openCrudForm,
  save: saveItem,
  deleteOpen,
  deleting,
  removing,
  deleteError,
  askDelete,
  remove,
} = useCrudCollection({
  label,
  fetch: api.webhookUrls,
  create: api.createWebhookUrl,
  update: api.updateWebhookUrl,
  remove: api.deleteWebhookUrl,
});

function openForm(item?: WebhookUrl) {
  Object.assign(state, {
    url: item?.url ?? "",
    description: item?.description ?? "",
    is_active: item?.is_active ?? true,
    events: item ? [...item.events] : webhookEvents.map((event) => event.value),
  });
  openCrudForm(item);
}

function toggleEvent(value: string, checked: boolean | "indeterminate") {
  state.events = state.events.filter((event) => event !== value);
  if (checked === true) state.events.push(value);
}

function save() {
  return saveItem({
    ...state,
    url: state.url.trim(),
    description: state.description?.trim() || null,
    events: [...state.events],
  });
}
</script>

<template>
  <section class="space-y-6" aria-labelledby="webhook-heading">
    <SettingsListHeader
      heading-id="webhook-heading"
      :title="'Webhook'"
      description="支出・収入の登録時や予算超過時に通知を送信するWebhook URLを管理します。"
      :count="loading || loadError ? null : items.length"
      :add-disabled="loading || !!loadError"
      @add="openForm()"
    />

    <SettingsCollectionState
      :label="label"
      :loading="loading"
      :error="loadError"
      :empty="!items.length"
      empty-icon="i-lucide-webhook"
      empty-description="通知先のURLを登録すると、イベント発生時にWebhookを送信できます。"
      add-label="Webhook URLを追加"
      skeleton-class="h-32"
      @retry="load"
      @add="openForm()"
    >
      <ul
        class="min-w-0 divide-y divide-default overflow-hidden rounded-lg border border-default"
        aria-label="登録済みのWebhook URL一覧"
      >
        <li v-for="item in items" :key="item.id" class="flex min-w-0 items-start gap-3 p-4">
          <div class="min-w-0 flex-1 space-y-2">
            <p class="font-medium break-all text-default">
              {{ item.url }}
            </p>
            <p v-if="item.description" class="text-sm break-words whitespace-pre-wrap text-muted">
              {{ item.description }}
            </p>
            <div class="flex flex-wrap gap-2">
              <UBadge color="neutral" :variant="item.is_active ? 'outline' : 'soft'">
                {{ item.is_active ? "有効" : "無効" }}
              </UBadge>
              <UBadge v-for="event in item.events" :key="event" color="neutral" variant="soft">
                {{ webhookEvents.find((option) => option.value === event)?.label ?? event }}
              </UBadge>
            </div>
          </div>
          <RowActionsMenu :label="item.url" @edit="openForm(item)" @delete="askDelete(item)" />
        </li>
      </ul>
    </SettingsCollectionState>

    <SettingsFormModal
      v-model:open="formOpen"
      :label="label"
      :editing="!!editingId"
      description="支出・収入の登録時や予算超過時に通知を送信するURLを設定します。"
      form-id="webhook-form"
      :state="state"
      :validate="validateWebhook"
      :saving="saving"
      :error="formError"
      @submit="save"
    >
      <UFormField name="url" label="Webhook URL" required>
        <UInput
          v-model="state.url"
          type="url"
          placeholder="https://example.com/webhook"
          class="w-full"
          autofocus
        />
      </UFormField>
      <UFormField name="description" label="説明（任意）">
        <UTextarea
          v-model="state.description"
          :maxlength="500"
          placeholder="通知先の用途やメモを入力"
          class="w-full"
        />
      </UFormField>
      <UFormField name="is_active">
        <USwitch v-model="state.is_active" label="通知を有効にする" />
      </UFormField>
      <UFormField name="events" required>
        <fieldset class="space-y-3">
          <legend class="mb-3 text-sm font-medium">通知するイベント</legend>
          <UCheckbox
            v-for="event in webhookEvents"
            :key="event.value"
            :model-value="state.events.includes(event.value)"
            :label="event.label"
            @update:model-value="toggleEvent(event.value, $event)"
          />
        </fieldset>
      </UFormField>
    </SettingsFormModal>

    <SettingsDeleteDialog
      v-model:open="deleteOpen"
      :label="label"
      :target="deleting?.url ?? ''"
      :busy="removing"
      :error="deleteError"
      description="この通知先を削除します。この操作は取り消せません。"
      @confirm="remove"
    />
  </section>
</template>
