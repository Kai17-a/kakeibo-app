<script lang="ts">
  import * as Card from '$lib/components/ui/card';
  import { paymentMethodBalanceTrend } from '../../domain/summaries';
  import { formatYen } from '../../format';
  import type { Expense, Income, PaymentMethod } from '../../types';

  interface Props {
    incomes: Income[];
    expenses: Expense[];
    paymentMethods: PaymentMethod[];
    year: string;
  }

  let { incomes, expenses, paymentMethods, year }: Props = $props();
  const months = $derived(paymentMethodBalanceTrend(incomes, expenses, paymentMethods, year));
  const chartColors = ['bg-chart-1', 'bg-chart-2', 'bg-chart-3', 'bg-chart-4', 'bg-chart-5'];
  const visibleMethods = $derived(months[0]?.values ?? []);
  const maxPositive = $derived(
    Math.max(0, ...months.flatMap((item) => item.values.map((value) => value.balance))),
  );
  const maxNegative = $derived(
    Math.max(0, ...months.flatMap((item) => item.values.map((value) => -value.balance))),
  );
  const chartRange = $derived(Math.max(1, maxPositive + maxNegative));
  const positiveHeight = $derived((maxPositive / chartRange) * 100);
</script>

<Card.Root>
  <Card.Header>
    <Card.Title>資産残高推移</Card.Title>
    <Card.Description>支払方法ごとの月末残高</Card.Description>
  </Card.Header>
  <Card.Content>
    {#if visibleMethods.length}
      <div class="relative flex h-72 gap-2">
        {#each months as item (item.month)}
          <div class="flex h-full flex-1 flex-col">
            <div class="relative h-[calc(100%_-_2rem)]">
              <div
                class="pointer-events-none absolute inset-x-0 border-t border-foreground/40"
                style:top={`${positiveHeight}%`}
                aria-hidden="true"
              ></div>
              {#each item.values as value, index (value.id)}
                <div
                  class={['absolute', chartColors[index % chartColors.length]]}
                  style:left={`${(index / item.values.length) * 100}%`}
                  style:width={`${100 / item.values.length}%`}
                  style:top={value.balance >= 0
                    ? `${positiveHeight - (value.balance / chartRange) * 100}%`
                    : `${positiveHeight}%`}
                  style:height={`${(Math.abs(value.balance) / chartRange) * 100}%`}
                  title={`${value.name}: ${formatYen(value.balance)}`}
                ></div>
              {/each}
            </div>
            <span class="mt-2 text-center text-xs">{item.month}月</span>
          </div>
        {/each}
      </div>
      <div class="mt-4 flex flex-wrap gap-x-4 gap-y-2 text-xs text-muted-foreground">
        {#each visibleMethods as method, index (method.id)}
          <span class="flex items-center gap-1.5">
            <span class={['size-2.5', chartColors[index % chartColors.length]]}></span>
            {method.name}
          </span>
        {/each}
      </div>
    {:else}
      <p class="py-24 text-center text-sm text-muted-foreground">
        初期残高が設定された支払方法はありません。
      </p>
    {/if}
  </Card.Content>
</Card.Root>
