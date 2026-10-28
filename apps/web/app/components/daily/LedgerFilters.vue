<script setup lang="ts">
import type { Category, PaymentMethod } from "~/types/settings";

defineProps<{
  categories: Category[];
  paymentMethods: PaymentMethod[];
}>();

/** The description keyword; an empty category or payment method id means "all". */
const keyword = defineModel<string>("keyword", { required: true });
const categoryId = defineModel<string>("categoryId", { required: true });
const paymentMethodId = defineModel<string>("paymentMethodId", { required: true });

// Select items can't use "" as a value, so "all" is represented by a sentinel in the select only.
const ALL_FILTER_VALUE = "__ALL__";
const fromSelectValue = (value: unknown) =>
  value == null || value === ALL_FILTER_VALUE ? "" : String(value);

function clear() {
  keyword.value = "";
  categoryId.value = "";
  paymentMethodId.value = "";
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-3 border-b border-default py-4">
    <AppFormField label="備考を検索" class="min-w-48 flex-1">
      <AppInput v-model="keyword" type="search" placeholder="キーワードを入力" class="w-full" />
    </AppFormField>
    <AppFormField label="カテゴリ" class="min-w-40 flex-1 sm:flex-none">
      <AppSelect
        :model-value="categoryId || ALL_FILTER_VALUE"
        :items="[
          { label: 'すべて', value: ALL_FILTER_VALUE },
          ...categories.map((category) => ({ label: category.name, value: category.id })),
        ]"
        class="w-full sm:w-44"
        @update:model-value="categoryId = fromSelectValue($event)"
      />
    </AppFormField>
    <AppFormField label="支払方法" class="min-w-40 flex-1 sm:flex-none">
      <AppSelect
        :model-value="paymentMethodId || ALL_FILTER_VALUE"
        :items="[
          { label: 'すべて', value: ALL_FILTER_VALUE },
          ...paymentMethods.map((method) => ({ label: method.name, value: method.id })),
        ]"
        class="w-full sm:w-44"
        @update:model-value="paymentMethodId = fromSelectValue($event)"
      />
    </AppFormField>
    <AppButton color="neutral" variant="outline" @click="clear"> 条件をクリア </AppButton>
  </div>
</template>
