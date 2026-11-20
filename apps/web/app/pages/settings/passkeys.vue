<script setup lang="ts">
import type { Passkey } from "~/composables/usePasskeys";
import { passkeyErrorMessage, passkeySupported } from "~/utils/webauthn";

useSeoMeta({ title: "パスキー設定" });

const auth = useAuth();
const api = usePasskeys();

const items = ref<Passkey[]>([]);
const loading = ref(true);
const loadError = ref("");
const canUsePasskey = ref(true);

const name = ref("");
const registering = ref(false);
const registerError = ref("");

const deleteTarget = ref<Passkey | null>(null);
const deleteOpen = ref(false);
const deleting = ref(false);
const deleteError = ref("");

const serverSupportsPasskey = computed(() => !!auth.session.value?.passkey);
const unavailableReason = computed(() =>
  auth.session.value?.mode === "local"
    ? "KAKEIBO_PUBLIC_URL に、IPアドレスではないドメインのURLを設定してください。"
    : "ログイン機能が無効になっています（KAKEIBO_AUTH_MODE=local で有効になります）。",
);

const dateFormat = new Intl.DateTimeFormat("ja-JP", { dateStyle: "medium" });
function formatEpoch(seconds: number) {
  return dateFormat.format(new Date(seconds * 1000));
}

async function load() {
  loading.value = true;
  loadError.value = "";
  try {
    items.value = await api.list();
  } catch (cause) {
    loadError.value = apiErrorMessage(cause);
  } finally {
    loading.value = false;
  }
}

async function register() {
  const label = name.value.trim() || "このデバイス";
  registering.value = true;
  registerError.value = "";
  try {
    const created = await api.register(label);
    items.value = [...items.value, created];
    name.value = "";
  } catch (cause) {
    registerError.value = passkeyErrorMessage(cause);
  } finally {
    registering.value = false;
  }
}

function askDelete(item: Passkey) {
  deleteTarget.value = item;
  deleteError.value = "";
  deleteOpen.value = true;
}

async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target) return;
  deleting.value = true;
  deleteError.value = "";
  try {
    await api.remove(target.id);
    items.value = items.value.filter((item) => item.id !== target.id);
    deleteOpen.value = false;
  } catch (cause) {
    deleteError.value = apiErrorMessage(cause);
  } finally {
    deleting.value = false;
  }
}

onMounted(() => {
  canUsePasskey.value = passkeySupported();
  if (serverSupportsPasskey.value) void load();
  else loading.value = false;
});
</script>

<template>
  <section class="space-y-6" aria-labelledby="passkey-heading">
    <div class="space-y-1">
      <h2
        id="passkey-heading"
        class="flex items-center gap-2 text-base font-semibold text-highlighted"
      >
        パスキー
        <UBadge
          v-if="!loading && !loadError && serverSupportsPasskey"
          color="neutral"
          variant="soft"
        >
          {{ items.length }}件
        </UBadge>
      </h2>
      <p class="text-sm text-muted">
        登録すると、パスワードの代わりに Face ID や指紋、画面ロックでログインできます。
      </p>
    </div>

    <UAlert
      v-if="!serverSupportsPasskey"
      color="neutral"
      variant="subtle"
      icon="i-lucide-info"
      title="このサーバーではパスキーを使えません"
      :description="unavailableReason"
    />

    <template v-else>
      <UAlert
        v-if="!canUsePasskey"
        color="warning"
        variant="subtle"
        icon="i-lucide-triangle-alert"
        title="このブラウザではパスキーを登録できません"
        description="HTTPS で開いているか、ブラウザがパスキーに対応しているかを確認してください。"
      />
      <form v-else class="flex flex-col gap-3 sm:flex-row sm:items-end" @submit.prevent="register">
        <UFormField label="名前" description="あとで見分けられる名前（例: iPhone）" class="flex-1">
          <UInput
            v-model="name"
            placeholder="このデバイス"
            :maxlength="64"
            :disabled="registering"
            class="w-full"
          />
        </UFormField>
        <UButton type="submit" icon="i-lucide-plus" :loading="registering" class="justify-center">
          パスキーを追加
        </UButton>
      </form>
      <UAlert
        v-if="registerError"
        color="error"
        variant="subtle"
        role="alert"
        title="登録できませんでした"
        :description="registerError"
      />

      <SettingsCollectionState
        label="パスキー"
        :loading="loading"
        :error="loadError"
        :empty="!items.length"
        empty-icon="i-lucide-key-round"
        empty-description="上のフォームからパスキーを追加すると、ここに一覧が表示されます"
        add-label="パスキーを追加"
        :show-add="false"
        @retry="load"
      >
        <ul class="divide-y divide-default border-y border-default" aria-label="パスキー一覧">
          <li
            v-for="item in items"
            :key="item.id"
            class="flex items-center gap-3 px-3 py-3 text-sm"
          >
            <UIcon name="i-lucide-key-round" class="size-5 shrink-0 text-muted" />
            <div class="min-w-0 flex-1">
              <p class="font-medium break-words text-default">{{ item.name }}</p>
              <p class="text-xs text-muted">
                登録 {{ formatEpoch(item.created_at) }} ・
                {{ item.last_used_at ? `最終使用 ${formatEpoch(item.last_used_at)}` : "未使用" }}
              </p>
            </div>
            <UButton
              icon="i-lucide-trash-2"
              color="neutral"
              variant="ghost"
              :aria-label="`${item.name}を削除`"
              class="max-sm:min-h-[44px] max-sm:min-w-[44px] max-sm:justify-center"
              @click="askDelete(item)"
            />
          </li>
        </ul>
      </SettingsCollectionState>
    </template>

    <SettingsDeleteDialog
      v-model:open="deleteOpen"
      label="パスキー"
      :target="deleteTarget?.name ?? ''"
      description="このパスキーではログインできなくなります。端末に残ったパスキーは端末の設定から削除してください。"
      :busy="deleting"
      :error="deleteError"
      @confirm="confirmDelete"
    />
  </section>
</template>
