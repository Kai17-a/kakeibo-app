<script setup lang="ts">
import type { PaymentMethod } from "~/types/settings";
import type { Transfer, TransferInput } from "~/types/transactions";
import { dateStringToCalendarDate } from "~/utils/calendar-date";

const props = defineProps<{
  open: boolean;
  paymentMethods: PaymentMethod[];
  saving: boolean;
  initialDate: string;
  initialTransfer?: Transfer | null;
  onSubmit: (input: TransferInput, keepOpen: boolean) => Promise<boolean>;
}>();

const emit = defineEmits<{ "update:open": [value: boolean] }>();
const editing = computed(() => Boolean(props.initialTransfer));
const unavailable = computed(() => props.paymentMethods.length < 2);
const paymentMethodOptions = computed(() =>
  props.paymentMethods.map((method) => ({ label: method.name, value: method.id })),
);
const state = reactive({
  date: props.initialTransfer?.transaction_date ?? props.initialDate,
  amount: String(props.initialTransfer?.amount ?? ""),
  fromPaymentMethodId:
    props.initialTransfer?.from_payment_method_id ?? props.paymentMethods[0]?.id ?? "",
  toPaymentMethodId:
    props.initialTransfer?.to_payment_method_id ?? props.paymentMethods[1]?.id ?? "",
  description: props.initialTransfer?.description ?? "",
});
const dateError = ref("");
const fromPaymentMethodError = ref("");
const toPaymentMethodError = ref("");

watch(
  () => state.date,
  (value) => {
    if (dateStringToCalendarDate(value)) dateError.value = "";
  },
);

async function handleSubmit(event: SubmitEvent) {
  dateError.value = dateStringToCalendarDate(state.date) ? "" : "日付を選択してください。";
  fromPaymentMethodError.value = state.fromPaymentMethodId ? "" : "移動元を選択してください。";
  toPaymentMethodError.value = state.toPaymentMethodId ? "" : "移動先を選択してください。";
  if (
    dateError.value ||
    unavailable.value ||
    fromPaymentMethodError.value ||
    toPaymentMethodError.value ||
    state.fromPaymentMethodId === state.toPaymentMethodId
  )
    return;

  const keepOpen = (event.submitter as HTMLButtonElement | null)?.value === "continue";
  const saved = await props.onSubmit(
    {
      transaction_date: state.date,
      amount: state.amount,
      from_payment_method_id: state.fromPaymentMethodId,
      to_payment_method_id: state.toPaymentMethodId,
      description: state.description || null,
    },
    keepOpen,
  );
  if (saved && keepOpen) {
    state.amount = "";
    state.description = "";
  }
}
</script>

<template>
  <UModal
    :open="open"
    :title="editing ? '振替を編集' : '振替を登録'"
    description="口座間の資金移動を入力してください。"
    :dismissible="!saving"
    :close="!saving"
    :ui="{ content: 'max-w-xl', body: 'sm:p-6', footer: 'flex-wrap justify-end gap-2' }"
    @update:open="(value) => emit('update:open', value)"
  >
    <template #body>
      <p v-if="unavailable" class="mb-5 text-sm text-muted">
        振替を登録するには、
        <NuxtLink
          to="/settings/payment-methods"
          class="text-primary underline-offset-2 hover:underline"
        >
          支払方法を2件以上登録してください。
        </NuxtLink>
      </p>
      <form
        id="transfer-form"
        class="grid grid-cols-1 gap-5 sm:grid-cols-2"
        @submit.prevent="handleSubmit"
      >
        <UFormField name="date" label="日付" required :error="dateError || undefined">
          <DatePicker v-model="state.date" required class="w-full" />
        </UFormField>
        <UFormField label="金額" required>
          <UInput
            :model-value="state.amount"
            type="number"
            min="1"
            step="1"
            required
            class="w-full"
            @update:model-value="(value) => (state.amount = String(value ?? ''))"
          />
        </UFormField>
        <UFormField label="移動元" required :error="fromPaymentMethodError || undefined">
          <USelect
            v-model="state.fromPaymentMethodId"
            :items="paymentMethodOptions"
            class="w-full"
          />
        </UFormField>
        <UFormField
          label="移動先"
          required
          :error="
            toPaymentMethodError ||
            (state.fromPaymentMethodId === state.toPaymentMethodId
              ? '移動元と異なる支払方法を選択してください。'
              : undefined)
          "
        >
          <USelect v-model="state.toPaymentMethodId" :items="paymentMethodOptions" class="w-full" />
        </UFormField>
        <UFormField label="メモ（任意）" class="sm:col-span-2">
          <UTextarea v-model="state.description" class="w-full" />
        </UFormField>
      </form>
    </template>
    <template #footer>
      <UButton
        color="neutral"
        variant="outline"
        :disabled="saving"
        @click="emit('update:open', false)"
      >
        キャンセル
      </UButton>
      <UButton
        v-if="!editing"
        type="submit"
        form="transfer-form"
        value="continue"
        color="neutral"
        variant="outline"
        :loading="saving"
        :disabled="saving || unavailable"
      >
        登録して続ける
      </UButton>
      <UButton
        type="submit"
        form="transfer-form"
        :loading="saving"
        :disabled="saving || unavailable"
      >
        {{ editing ? "更新する" : "登録する" }}
      </UButton>
    </template>
  </UModal>
</template>
