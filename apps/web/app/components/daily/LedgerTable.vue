<script setup lang="ts" generic="T extends Expense | Income">
import type { Expense, Income } from "~/types/transactions";
import { formatDate, formatSignedCurrency } from "~/utils/format";

const props = defineProps<{
  kind: "expense" | "income";
  /** The rows to show, already filtered and sorted. */
  items: T[];
  /** Shown instead of the rows when `items` is empty. */
  emptyText: string;
  categoryNames: Map<string, string>;
  paymentNames: Map<string, string>;
}>();

const emit = defineEmits<{
  edit: [item: T];
  delete: [item: T];
}>();

function label(item: T) {
  return (
    item.description ||
    props.categoryNames.get(item.category_id) ||
    (props.kind === "expense" ? "支出" : "収入")
  );
}

/** Incomes may have no payment method; expenses always have one. */
function paymentName(item: T) {
  if (props.kind === "expense")
    return item.payment_method_id ? props.paymentNames.get(item.payment_method_id) : undefined;
  return item.payment_method_id ? (props.paymentNames.get(item.payment_method_id) ?? "—") : "—";
}
</script>

<template>
  <div class="max-h-[38rem] overflow-auto">
    <ul class="space-y-2 sm:hidden">
      <li v-for="item in items" :key="item.id" class="rounded-md border border-default p-3">
        <div class="flex items-start justify-between gap-3 text-sm">
          <div class="min-w-0 break-words">
            <p class="text-xs text-muted">{{ formatDate(item.transaction_date) }}</p>
            <p>{{ categoryNames.get(item.category_id) }}</p>
          </div>
          <p
            class="shrink-0 text-right tabular-nums"
            :class="kind === 'income' && Number(item.amount) > 0 ? 'text-primary' : undefined"
          >
            {{ formatSignedCurrency(item.amount, kind === "expense" ? "negative" : "positive") }}
          </p>
        </div>
        <div class="mt-2 flex items-center justify-between gap-2">
          <p class="min-w-0 text-xs break-words text-muted">
            {{ paymentName(item) }}<span v-if="item.description"> · {{ item.description }}</span>
          </p>
          <RowActionsMenu
            :label="`${formatDate(item.transaction_date)} ${label(item)}`"
            @edit="emit('edit', item)"
            @delete="emit('delete', item)"
          />
        </div>
      </li>
    </ul>
    <table class="hidden w-full min-w-[800px] text-sm sm:table">
      <thead class="sticky top-0 bg-default">
        <tr class="border-b border-default">
          <th class="w-32 px-3 py-2 text-left text-xs font-medium whitespace-nowrap text-muted">
            日付
          </th>
          <th class="min-w-28 px-3 py-2 text-left text-xs font-medium text-muted">摘要</th>
          <th class="min-w-28 px-3 py-2 text-left text-xs font-medium text-muted">支払方法</th>
          <th class="w-32 px-3 py-2 text-right text-xs font-medium whitespace-nowrap text-muted">
            金額
          </th>
          <th class="min-w-40 px-3 py-2 text-left text-xs font-medium text-muted">メモ</th>
          <th class="w-36 px-3 py-2 text-right">
            <span class="sr-only">操作</span>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="item in items"
          :key="item.id"
          class="border-t border-default transition-colors hover:bg-elevated/50"
        >
          <td class="px-3 py-2 whitespace-nowrap">
            {{ formatDate(item.transaction_date) }}
          </td>
          <td class="px-3 py-2">
            {{ categoryNames.get(item.category_id) }}
          </td>
          <td class="px-3 py-2">
            {{ paymentName(item) }}
          </td>
          <td
            :class="[
              'px-3 py-2 text-right whitespace-nowrap tabular-nums',
              kind === 'income'
                ? Number(item.amount) > 0
                  ? 'text-primary'
                  : 'text-default'
                : undefined,
            ]"
          >
            {{ formatSignedCurrency(item.amount, kind === "income" ? "positive" : "negative") }}
          </td>
          <td class="px-3 py-2">
            {{ item.description ?? "" }}
          </td>
          <td class="px-3 py-2 text-right">
            <RowActionsMenu
              :label="`${formatDate(item.transaction_date)} ${label(item)}`"
              @edit="emit('edit', item)"
              @delete="emit('delete', item)"
            />
          </td>
        </tr>
      </tbody>
    </table>
    <p v-if="!items.length" class="py-12 text-center text-sm text-muted">
      {{ emptyText }}
    </p>
  </div>
</template>
