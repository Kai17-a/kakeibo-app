<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { categoryMonthlyTotals } from '../../domain/summaries';
  import { formatYen } from '../../format';
  import type { Expense, ExpenseCategory } from '../../types';

  interface Props {
    expenses: Expense[];
    categories: ExpenseCategory[];
    year: string;
  }

  let { expenses, categories, year }: Props = $props();
  const months = $derived(categoryMonthlyTotals(expenses, categories, year));
  const chartMax = $derived(Math.max(1, ...months.map((item) => item.total)));
  const chartColors = ['bg-chart-1', 'bg-chart-2', 'bg-chart-3', 'bg-chart-4', 'bg-chart-5'];
  const visibleCategories = $derived(months[0]?.values ?? []);
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>カテゴリ別月次推移</Card.Title>
    <Card.Description>月ごとの支出構成</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if visibleCategories.length}
      <div class="flex h-72 items-end gap-2 border-b">
        {#each months as item (item.month)}
          <div class="flex h-full flex-1 flex-col justify-end">
            <div class="flex h-[calc(100%_-_2rem)] flex-col-reverse justify-start">
              {#each item.values as value, index (value.id)}
                {#if value.total}
                  <div
                    class={['w-full', chartColors[index % chartColors.length]]}
                    style:height={`${(value.total / chartMax) * 100}%`}
                    title={`${value.name}: ${formatYen(value.total)}`}
                  ></div>
                {/if}
              {/each}
            </div>
            <span class="mt-2 text-center text-xs">{item.month}月</span>
          </div>
        {/each}
      </div>
      <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted-foreground">
        {#each visibleCategories as category, index (category.id)}
          <span class="flex items-center gap-1.5">
            <span class={['size-2.5', chartColors[index % chartColors.length]]}></span>
            {category.name}
          </span>
        {/each}
      </div>
    {:else}
      <p class="py-24 text-center text-sm text-muted-foreground">この年の支出はありません。</p>
    {/if}
  </Card.Content>
</Card.Root>
