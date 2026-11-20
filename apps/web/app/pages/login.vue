<script setup lang="ts">
import { loginErrorMessage, safeRedirectPath } from "~/utils/auth";
import { passkeyErrorMessage, passkeySupported } from "~/utils/webauthn";

definePageMeta({ layout: false });
useSeoMeta({ title: "ログイン" });

const auth = useAuth();
const route = useRoute();

const state = reactive({ username: "", password: "" });
const submitting = ref(false);
const error = ref("");
// Read after mounting: the browser API is not available while the page is prerendered.
const canUsePasskey = ref(false);
onMounted(() => {
  canUsePasskey.value = passkeySupported();
});
const showPasskey = computed(() => canUsePasskey.value && !!auth.session.value?.passkey);

function validate(value: typeof state) {
  const errors: { name: string; message: string }[] = [];
  if (!value.username.trim())
    errors.push({ name: "username", message: "ユーザー名を入力してください" });
  if (!value.password) errors.push({ name: "password", message: "パスワードを入力してください" });
  return errors;
}

async function loginWithPasskey() {
  submitting.value = true;
  error.value = "";
  try {
    await auth.loginWithPasskey();
    await navigateTo(safeRedirectPath(route.query.redirect), { replace: true });
  } catch (cause) {
    error.value = passkeyErrorMessage(cause);
  } finally {
    submitting.value = false;
  }
}

async function submit() {
  submitting.value = true;
  error.value = "";
  try {
    await auth.login(state.username.trim(), state.password);
    await navigateTo(safeRedirectPath(route.query.redirect), { replace: true });
  } catch (cause) {
    const status =
      cause && typeof cause === "object" && "statusCode" in cause
        ? (cause.statusCode as number | undefined)
        : undefined;
    error.value = loginErrorMessage(status);
    state.password = "";
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <main class="flex min-h-dvh items-center justify-center bg-default px-4 py-10">
    <UCard class="w-full max-w-sm" :ui="{ body: 'space-y-6 sm:p-8' }">
      <div class="flex items-center gap-3">
        <AppLogo class="size-9 shrink-0" />
        <h1 class="text-xl font-semibold text-highlighted">Kakeibo にログイン</h1>
      </div>

      <UForm
        :state="state"
        :validate="validate"
        :disabled="submitting"
        :validate-on="['blur']"
        class="space-y-5"
        @submit="submit"
      >
        <UAlert v-if="error" color="error" variant="subtle" :description="error" role="alert" />
        <UFormField label="ユーザー名" name="username">
          <UInput
            v-model="state.username"
            autocomplete="username"
            autocapitalize="none"
            :spellcheck="false"
            autofocus
            size="xl"
            class="w-full"
          />
        </UFormField>
        <UFormField label="パスワード" name="password">
          <UInput
            v-model="state.password"
            type="password"
            autocomplete="current-password"
            size="xl"
            class="w-full"
          />
        </UFormField>
        <UButton type="submit" block size="xl" :loading="submitting">ログイン</UButton>
      </UForm>

      <template v-if="showPasskey">
        <USeparator label="または" />
        <UButton
          icon="i-lucide-key-round"
          color="neutral"
          variant="outline"
          block
          size="xl"
          :disabled="submitting"
          @click="loginWithPasskey"
        >
          パスキーでログイン
        </UButton>
      </template>
    </UCard>
  </main>
</template>
