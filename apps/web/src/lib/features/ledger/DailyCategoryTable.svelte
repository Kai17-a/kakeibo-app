<script lang="ts">
  import * as Empty from '$lib/components/ui/empty';
  import { dailyCategoryTotals, sumAmounts } from '../../domain/summaries';
  import { formatDate } from '../../format';
  import type { Expense, ExpenseCategory } from '../../types';
  import { cn } from '../../utils';
  interface Props {
    month: string;
    expenses: Expense[];
    categories: ExpenseCategory[];
  }
  let { month, expenses, categories }: Props = $props();
  const days = $derived(dailyCategoryTotals(expenses, month));
  const headBase = 'px-3 py-2 text-xs font-medium tracking-wider text-muted-foreground uppercase';
  const dateHeadClass = cn(
    headBase,
    'sticky left-0 w-32 min-w-32 bg-muted text-left whitespace-nowrap',
  );
  const categoryHeadClass = cn(headBase, 'min-w-24 text-right');
  const totalHeadClass = cn(headBase, 'sticky right-0 bg-muted text-right');
  const dateCellClass =
    'sticky left-0 w-32 min-w-32 bg-background px-3 py-2 text-left whitespace-nowrap';
  const totalCellClass = 'sticky right-0 bg-muted px-3 py-2 text-right font-bold';
  const dateFootClass =
    'sticky left-0 w-32 min-w-32 bg-muted px-3 py-3 text-left whitespace-nowrap';
  const totalFootClass = 'sticky right-0 bg-muted px-3 py-3 text-right font-bold';
</script>

<section class="p-4">
  <h3 class="mb-2 bg-muted px-3 py-2 text-sm font-bold">日ごとのカテゴリ別支出</h3>
  {#if categories.length}<div class="overflow-x-auto">
      <table class="w-full min-w-[760px] border-collapse text-sm">
        <thead class="bg-muted">
          <tr>
            <th class={dateHeadClass}>日付</th>
            {#each categories as category (category.id)}<th class={categoryHeadClass}>
                {category.name}
              </th>{/each}
            <th class={totalHeadClass}>合計</th>
          </tr>
        </thead>
        <tbody>
          {#each days as day (day.date)}<tr class="border-t">
              <th class={dateCellClass}>{formatDate(day.date)}</th>
              {#each categories as category (category.id)}<td class="px-3 py-2 text-right">
                  {day.values.get(category.id)?.toLocaleString('ja-JP') ?? ''}
                </td>{/each}
              <td class={totalCellClass}>{day.total ? day.total.toLocaleString('ja-JP') : ''}</td>
            </tr>{/each}
        </tbody>
        <tfoot class="border-t-2 bg-muted">
          <tr>
            <th class={dateFootClass}>合計</th>
            {#each categories as category (category.id)}<td class="px-3 py-3 text-right font-bold">
                {sumAmounts(
                  expenses.filter((item) => item.category_id === category.id),
                ).toLocaleString('ja-JP')}
              </td>{/each}
            <td class={totalFootClass}>{sumAmounts(expenses).toLocaleString('ja-JP')}</td>
          </tr>
        </tfoot>
      </table>
    </div>{:else}<Empty.Root>
      <Empty.Header>
        <Empty.Title>支出カテゴリがありません</Empty.Title><Empty.Description>
          カテゴリを追加すると日別集計が表示されます。
        </Empty.Description>
      </Empty.Header>
    </Empty.Root>{/if}
</section>
