<script lang="ts">
  interface Props { onIncome(): void; onExpense(): void; onRecurring(): void }
  let { onIncome, onExpense, onRecurring }: Props = $props();
</script>

<header class="border-b border-[#d8d5cc] bg-[#f8f6f0]/95">
  <div class="mx-auto flex max-w-7xl items-center justify-between px-5 py-4 lg:px-10">
    <a class="flex items-center gap-3" href="/" aria-label="Kakeibo ホーム"><span class="grid size-10 place-items-center rounded-xl bg-[#245c4a] text-lg font-black text-white">K</span><strong class="hidden font-serif text-xl tracking-tight sm:block">Kakeibo</strong></a>
    <div class="flex items-center gap-2">
      <button class="rounded-xl border border-[#c8c7bf] bg-white px-4 py-2 text-sm font-semibold hover:bg-[#eeece5]" onclick={onRecurring}>＋ 固定費</button>
      <button class="rounded-xl border border-[#c8c7bf] bg-white px-4 py-2 text-sm font-semibold hover:bg-[#eeece5]" onclick={onIncome}>＋ 収入</button>
      <button class="rounded-xl bg-[#245c4a] px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-[#194738]" onclick={onExpense}>＋ 支出</button>
    </div>
  </div>
</header>
