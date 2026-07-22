<script lang="ts">
  import { dailyCategoryTotals, sumAmounts } from '../../domain/summaries';
  import { formatDate } from '../../format';
  import type { Expense, ExpenseCategory } from '../../types';
  interface Props { month: string; expenses: Expense[]; categories: ExpenseCategory[] }
  let { month, expenses, categories }: Props = $props();
  const days = $derived(dailyCategoryTotals(expenses, month));
</script>

<section class="p-4"><h3 class="mb-2 bg-[#dce9df] px-3 py-2 text-sm font-bold">日ごとのカテゴリ別支出</h3>{#if categories.length}<div class="overflow-x-auto"><table class="w-full min-w-[760px] border-collapse text-sm"><thead class="bg-[#f5f5f1] text-xs"><tr><th class="sticky left-0 bg-[#f5f5f1] px-3 py-2 text-left">日付</th>{#each categories as category (category.id)}<th class="min-w-24 px-3 py-2 text-right">{category.name}</th>{/each}<th class="sticky right-0 bg-[#e9efe9] px-3 py-2 text-right">合計</th></tr></thead><tbody>{#each days as day (day.date)}<tr class="border-t"><th class="sticky left-0 bg-white px-3 py-2 text-left">{formatDate(day.date)}</th>{#each categories as category (category.id)}<td class="px-3 py-2 text-right">{day.values.get(category.id)?.toLocaleString('ja-JP') ?? ''}</td>{/each}<td class="sticky right-0 bg-[#f1f5f1] px-3 py-2 text-right font-bold">{day.total ? day.total.toLocaleString('ja-JP') : ''}</td></tr>{/each}</tbody><tfoot class="border-t-2 bg-[#e9efe9]"><tr><th class="sticky left-0 bg-[#e9efe9] px-3 py-3 text-left">合計</th>{#each categories as category (category.id)}<td class="px-3 py-3 text-right font-bold">{sumAmounts(expenses.filter((item) => item.category_id === category.id)).toLocaleString('ja-JP')}</td>{/each}<td class="sticky right-0 bg-[#dce8de] px-3 py-3 text-right font-bold">{sumAmounts(expenses).toLocaleString('ja-JP')}</td></tr></tfoot></table></div>{:else}<p class="py-12 text-center text-sm text-[#8a918d]">支出カテゴリがありません。</p>{/if}</section>
