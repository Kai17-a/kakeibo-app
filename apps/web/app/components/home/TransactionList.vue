<script setup lang="ts">
import type { Transaction } from "~/utils/summaries";
import type { TransactionDayGroup } from "~/utils/transactions";
import { formatDate, formatSignedCurrency } from "~/utils/format";

defineProps<{
  monthLabel: string;
  groups: TransactionDayGroup[];
  count: number;
  /** The primary text of a row (description, or the category when there is none). */
  labelOf: (item: Transaction) => string;
  /** The secondary text of a row (category and payment method). */
  metaOf: (item: Transaction) => string;
}>();

const emit = defineEmits<{
  create: [];
  edit: [item: Transaction];
  delete: [item: Transaction];
}>();

function actions(item: Transaction) {
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
  <UCard>
    <template #header>
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-semibold">明細</h2>
        <UBadge color="neutral" variant="outline"> {{ count }} 件 </UBadge>
      </div>
      <p class="text-sm text-muted">
        {{ monthLabel }}
      </p>
    </template>
    <div v-if="groups.length">
      <section
        v-for="group in groups"
        :key="group.date"
        class="border-b border-muted py-3 last:border-b-0"
      >
        <div class="mb-1 flex items-center justify-between gap-3 text-xs text-muted">
          <h3 class="font-medium text-toned">
            {{ formatDate(group.date) }}
          </h3>
          <span class="tabular-nums">当日計 {{ formatSignedCurrency(group.total) }}</span>
        </div>
        <ul>
          <li
            v-for="item in group.items"
            :key="item.id"
            class="flex min-w-0 items-center gap-3 rounded-md py-2 hover:bg-elevated/50"
          >
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-semibold">
                {{ labelOf(item) }}
              </p>
              <p class="truncate text-xs text-muted">
                {{ metaOf(item) }}
              </p>
            </div>
            <p
              :class="[
                'shrink-0 font-semibold tabular-nums',
                item.kind === 'income' && Number(item.amount) > 0 ? 'text-primary' : 'text-default',
              ]"
            >
              {{
                formatSignedCurrency(item.amount, item.kind === "income" ? "positive" : "negative")
              }}
            </p>
            <UDropdownMenu :items="actions(item)">
              <UButton
                icon="i-lucide-ellipsis"
                color="neutral"
                variant="ghost"
                size="sm"
                :aria-label="`${formatDate(item.transaction_date)} ${labelOf(item)}の操作`"
              />
            </UDropdownMenu>
          </li>
        </ul>
      </section>
    </div>
    <p v-else class="py-10 text-center text-sm text-muted">
      この月の明細はまだありません。収入または支出を記録すると、ここに表示されます。
      <UButton icon="i-lucide-plus" class="mx-auto mt-4 flex w-fit" @click="emit('create')">
        記録する
      </UButton>
    </p>
  </UCard>
</template>
