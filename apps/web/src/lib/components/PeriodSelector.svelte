<script lang="ts">
  export type SummaryView = 'monthly' | 'daily' | 'annual';
  interface Props { view: SummaryView; month: string; year: string; years: string[]; onchange(view: SummaryView): void; onmonth(value: string): void; onyear(value: string): void }
  let { view, month, year, years, onchange, onmonth, onyear }: Props = $props();
</script>

<section class="mb-8 flex flex-wrap items-end justify-between gap-5">
  <div><p class="mb-1 text-sm font-bold tracking-widest text-[#39705d]">OVERVIEW</p><h1 class="font-serif text-3xl font-semibold lg:text-4xl">{view === 'annual' ? `${year}年の家計` : view === 'daily' ? '日別カテゴリ集計' : '今月の家計'}</h1></div>
  <div class="flex flex-wrap items-end gap-3">
    <div class="flex rounded-xl border border-[#cbc9c0] bg-white p-1" aria-label="集計期間">
      {#each [['monthly', '月間'], ['daily', '日別集計'], ['annual', '年間']] as item (item[0])}<button class={['rounded-lg px-4 py-2 text-sm font-bold', view === item[0] ? 'bg-[#245c4a] text-white' : 'text-[#68746e] hover:bg-[#f0eee7]']} aria-pressed={view === item[0]} onclick={() => onchange(item[0] as SummaryView)}>{item[1]}</button>{/each}
    </div>
    {#if view === 'annual'}<label class="grid gap-1.5 text-xs font-bold text-[#59665f]">表示する年<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2 text-sm" value={year} onchange={(event) => onyear(event.currentTarget.value)}>{#each years as item (item)}<option value={item}>{item}年</option>{/each}</select></label>{:else}<label class="grid gap-1.5 text-xs font-bold text-[#59665f]">表示する月<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2 text-sm" type="month" value={month} onchange={(event) => onmonth(event.currentTarget.value)} /></label>{/if}
  </div>
</section>
