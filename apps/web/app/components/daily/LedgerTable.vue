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

function actions(item: T) {
  return [
    [
      { label: "編集", icon: "i-lucide-pencil", onSelect: () => emit("edit", item) },
      {
        label: "削除",
        icon: "i-lucide-trash-2",
        color: "error" as const,
        onSelect: () => emit("delete", item),
      },
    ],
  ];
}
</script>

<template>
  <div class="max-h-[38rem] overflow-auto">
    <table class="w-full min-w-[800px] text-sm">
      <thead class="sticky top-0 bg-default">
        <tr class="border-b border-default">
          <th
            class="w-32 px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
          >
            日付
          </th>
          <th
            class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
          >
            摘要
          </th>
          <th
            class="min-w-28 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
          >
            支払方法
          </th>
          <th
            class="w-32 px-3 py-2 text-right text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
          >
            金額
          </th>
          <th
            class="min-w-40 px-3 py-2 text-left text-xs font-medium tracking-wider text-muted uppercase"
          >
            備考
          </th>
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
            <UDropdownMenu :items="actions(item)">
              <UButton
                icon="i-lucide-ellipsis"
                color="neutral"
                variant="ghost"
                size="sm"
                :aria-label="`${formatDate(item.transaction_date)} ${label(item)}の操作`"
              />
            </UDropdownMenu>
          </td>
        </tr>
      </tbody>
    </table>
    <p v-if="!items.length" class="py-12 text-center text-sm text-muted">
      {{ emptyText }}
    </p>
  </div>
</template>
