<script lang="ts">
  import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
  import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
  import * as Field from '$lib/components/ui/field';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { shiftMonth } from '$lib/format';
  export type SummaryView = 'monthly' | 'daily' | 'annual';
  interface Props {
    view: SummaryView;
    month: string;
    year: string;
    years: string[];
    onchange(view: SummaryView): void;
    onmonth(value: string): void;
    onyear(value: string): void;
  }
  let { view, month, year, years, onchange, onmonth, onyear }: Props = $props();

  const shiftYear = (value: string, delta: number) => String(Number(value) + delta);
</script>

<section class="mb-8 flex flex-wrap items-end justify-between gap-5">
  <div>
    <p class="mb-1 text-sm font-bold tracking-widest text-muted-foreground">OVERVIEW</p>
    <h1 class="font-serif text-3xl font-semibold lg:text-4xl">
      {view === 'annual' ? `${year}年の家計` : view === 'daily' ? '日別カテゴリ集計' : '今月の家計'}
    </h1>
  </div>
  <div class="flex flex-wrap items-end gap-3">
    <ToggleGroup.Root
      type="single"
      value={view}
      variant="outline"
      aria-label="集計期間"
      onValueChange={(value) => value && onchange(value as SummaryView)}
    >
      <ToggleGroup.Item value="monthly">月間</ToggleGroup.Item>
      <ToggleGroup.Item value="daily">日別集計</ToggleGroup.Item>
      <ToggleGroup.Item value="annual">年間</ToggleGroup.Item>
    </ToggleGroup.Root>
    {#if view === 'annual'}
      <Field.Field class="w-auto">
        <Field.FieldLabel for="display-year">表示する年</Field.FieldLabel>
        <div class="flex w-fit items-center gap-2">
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="前年"
            onclick={() => onyear(shiftYear(year, -1))}
          >
            <ChevronLeftIcon data-icon="inline-start" />
          </Button>
          <NativeSelect.Root
            id="display-year"
            value={year}
            onchange={(event) => onyear(event.currentTarget.value)}
          >
            {#each years as item (item)}
              <NativeSelect.Option value={item}>{item}年</NativeSelect.Option>
            {/each}
          </NativeSelect.Root>
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="翌年"
            onclick={() => onyear(shiftYear(year, 1))}
          >
            <ChevronRightIcon data-icon="inline-end" />
          </Button>
        </div>
      </Field.Field>
    {:else}
      <Field.Field class="w-auto">
        <Field.FieldLabel for="display-month">表示する月</Field.FieldLabel>
        <div class="flex w-fit items-center gap-2">
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="前の月"
            onclick={() => onmonth(shiftMonth(month, -1))}
          >
            <ChevronLeftIcon data-icon="inline-start" />
          </Button>
          <Input
            id="display-month"
            type="month"
            value={month}
            onchange={(event) => onmonth(event.currentTarget.value)}
            class="w-44"
          />
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="次の月"
            onclick={() => onmonth(shiftMonth(month, 1))}
          >
            <ChevronRightIcon data-icon="inline-end" />
          </Button>
        </div>
      </Field.Field>
    {/if}
  </div>
</section>
