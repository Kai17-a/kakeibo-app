<script setup lang="ts">
import type { Category } from "~/types/settings";
import type { Expense } from "~/types/transactions";
import { formatCurrency, formatDate } from "~/utils/format";
import { dailyCategoryTotals, sumAmounts } from "~/utils/summaries";

const props = defineProps<{
  month: string;
  /** The month's expenses. */
  expenses: Expense[];
  categories: Category[];
}>();

const rows = computed(() => dailyCategoryTotals(props.expenses, props.month));
/** Only categories with spending this month get a column. */
const columns = computed(() =>
  props.categories.filter((category) =>
    props.expenses.some((item) => item.category_id === category.id),
  ),
);
const columnTotal = (categoryId: string) =>
  sumAmounts(props.expenses.filter((item) => item.category_id === categoryId));
</script>

<template>
  <div>
    <h2 class="mb-3 text-base font-semibold">日ごとのカテゴリ別支出</h2>
    <div v-if="columns.length" class="overflow-x-auto">
      <table class="w-full min-w-[760px] border-collapse text-sm">
        <thead class="bg-elevated">
          <tr>
            <th
              class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-2 text-left text-xs font-medium tracking-wider whitespace-nowrap text-muted uppercase"
            >
              日付
            </th>
            <th
              v-for="category in columns"
              :key="category.id"
              class="min-w-24 px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase"
            >
              {{ category.name }}
            </th>
            <th
              class="sticky right-0 bg-elevated px-3 py-2 text-right text-xs font-medium tracking-wider text-muted uppercase"
            >
              合計
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="day in rows" :key="day.date" class="border-t border-default">
            <th
              class="sticky left-0 w-32 min-w-32 bg-default px-3 py-2 text-left whitespace-nowrap"
            >
              {{ formatDate(day.date) }}
            </th>
            <td
              v-for="category in columns"
              :key="category.id"
              class="px-3 py-2 text-right tabular-nums"
            >
              {{ day.values.get(category.id) ? formatCurrency(day.values.get(category.id)!) : "" }}
            </td>
            <td class="sticky right-0 bg-elevated px-3 py-2 text-right font-bold tabular-nums">
              {{ day.total ? formatCurrency(day.total) : "" }}
            </td>
          </tr>
        </tbody>
        <tfoot class="border-t-2 border-default bg-elevated">
          <tr>
            <th
              class="sticky left-0 w-32 min-w-32 bg-elevated px-3 py-3 text-left whitespace-nowrap"
            >
              合計
            </th>
            <td
              v-for="category in columns"
              :key="category.id"
              class="px-3 py-3 text-right font-bold tabular-nums"
            >
              {{ formatCurrency(columnTotal(category.id)) }}
            </td>
            <td class="sticky right-0 bg-elevated px-3 py-3 text-right font-bold tabular-nums">
              {{ formatCurrency(sumAmounts(expenses)) }}
            </td>
          </tr>
        </tfoot>
      </table>
    </div>
    <p v-else class="py-10 text-center text-sm text-muted">この月の支出はまだありません。</p>
  </div>
</template>
