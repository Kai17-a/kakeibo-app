<script setup lang="ts">
import type { RecurringExpense, RecurringIncome } from "~/types/settings";
import type { ExchangeRatePreview } from "~/types/transactions";
import { formatCurrency } from "~/utils/format";

const props = defineProps<{
  /** Active recurring items; variable ones get a button to record this month's amount. */
  recurringExpenses: RecurringExpense[];
  recurringIncomes: RecurringIncome[];
  /** This month's JPY conversion of fixed USD recurring expenses, keyed by recurring expense id. */
  usdPreviews: Map<string, ExchangeRatePreview>;
}>();

const emit = defineEmits<{
  registerExpense: [item: RecurringExpense];
  registerIncome: [item: RecurringIncome];
}>();

const fixedExpenses = computed(() => props.recurringExpenses.filter((item) => !item.is_variable));
const variableExpenses = computed(() => props.recurringExpenses.filter((item) => item.is_variable));
const fixedIncomes = computed(() => props.recurringIncomes.filter((item) => !item.is_variable));
const variableIncomes = computed(() => props.recurringIncomes.filter((item) => item.is_variable));
const hasAny = computed(() => props.recurringExpenses.length || props.recurringIncomes.length);
</script>

<template>
  <UCard v-if="hasAny" class="bg-default">
    <template #header>
      <h2 class="text-lg font-semibold">定期の収支</h2>
    </template>
    <ul class="divide-y divide-default">
      <li
        v-for="item in fixedExpenses"
        :key="item.id"
        class="flex items-center justify-between py-3 text-sm"
      >
        <span>
          <b class="block">{{ item.name }}</b>
          <small class="text-muted">毎月 {{ item.payment_day }} 日</small>
        </span>
        <b class="text-right">
          <template v-if="item.currency_code === 'USD'">
            <span v-if="usdPreviews.get(item.id)" class="block">
              {{ formatCurrency(usdPreviews.get(item.id)!.converted_amount) }}
            </span>
            <span v-else class="font-normal"
              >USD {{ item.foreign_amount ?? item.amount }}（換算待ち）</span
            >
          </template>
          <template v-else>
            {{ formatCurrency(item.amount) }}
          </template>
        </b>
      </li>
    </ul>
    <template v-if="variableExpenses.length">
      <h3 class="mt-4 text-xs font-semibold tracking-widest text-muted uppercase">
        準固定費（金額変動）
      </h3>
      <ul class="mt-2 divide-y divide-default">
        <li
          v-for="item in variableExpenses"
          :key="item.id"
          class="flex items-center justify-between py-3 text-sm"
        >
          <span>
            <b class="block">{{ item.name }}</b>
            <small class="text-muted"
              >毎月 {{ item.payment_day }} 日 · 目安
              {{
                item.currency_code === "USD"
                  ? `USD ${item.foreign_amount ?? item.amount}`
                  : formatCurrency(item.amount)
              }}</small
            >
          </span>
          <UButton
            color="neutral"
            variant="outline"
            size="sm"
            :aria-label="`${item.name}の今月分を登録`"
            @click="emit('registerExpense', item)"
          >
            今月分を登録
          </UButton>
        </li>
      </ul>
    </template>
    <h3
      v-if="recurringIncomes.length"
      class="mt-5 border-t border-default pt-5 text-sm font-semibold"
    >
      収入
    </h3>
    <ul class="divide-y divide-default">
      <li
        v-for="item in fixedIncomes"
        :key="item.id"
        class="flex items-center justify-between py-3 text-sm"
      >
        <span>
          <b class="block">{{ item.name }}</b>
          <small class="text-muted">毎月 {{ item.payment_day }} 日</small>
        </span>
        <b>{{ formatCurrency(item.amount) }}</b>
      </li>
    </ul>
    <template v-if="variableIncomes.length">
      <h3 class="mt-4 text-xs font-semibold tracking-widest text-muted uppercase">
        準固定収入（金額変動）
      </h3>
      <ul class="mt-2 divide-y divide-default">
        <li
          v-for="item in variableIncomes"
          :key="item.id"
          class="flex items-center justify-between py-3 text-sm"
        >
          <span>
            <b class="block">{{ item.name }}</b>
            <small class="text-muted"
              >毎月 {{ item.payment_day }} 日 · 目安 {{ formatCurrency(item.amount) }}</small
            >
          </span>
          <UButton
            color="neutral"
            variant="outline"
            size="sm"
            :aria-label="`${item.name}の今月分を登録`"
            @click="emit('registerIncome', item)"
          >
            今月分を登録
          </UButton>
        </li>
      </ul>
    </template>
  </UCard>
  <UCard v-else class="bg-default">
    <template #header>
      <h2 class="text-lg font-semibold">定期の収支</h2>
    </template>
    <p class="mt-2 text-sm text-muted">
      定期的な収支はまだ設定されていません。
      <NuxtLink to="/settings/recurring-expenses" class="font-medium text-primary hover:underline">
        定期の収支を設定
      </NuxtLink>
    </p>
  </UCard>
</template>
