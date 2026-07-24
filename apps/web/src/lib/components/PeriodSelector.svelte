<script lang="ts">
  import * as Field from '$lib/components/ui/field';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import * as ToggleGroup from '$lib/components/ui/toggle-group';
  import { Input } from '$lib/components/ui/input';
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
      <Field.Field>
        <Field.FieldLabel for="display-year">表示する年</Field.FieldLabel>
        <NativeSelect.Root
          id="display-year"
          value={year}
          onchange={(event) => onyear(event.currentTarget.value)}
        >
          {#each years as item (item)}
            <NativeSelect.Option value={item}>{item}年</NativeSelect.Option>
          {/each}
        </NativeSelect.Root>
      </Field.Field>
    {:else}
      <Field.Field>
        <Field.FieldLabel for="display-month">表示する月</Field.FieldLabel>
        <Input
          id="display-month"
          type="month"
          value={month}
          onchange={(event) => onmonth(event.currentTarget.value)}
        />
      </Field.Field>
    {/if}
  </div>
</section>
