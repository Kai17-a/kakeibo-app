<script lang="ts">
  import type { ExpenseCategory, ExpenseInput, IncomeCategory, IncomeInput, PaymentMethod } from '../../types';
  interface Props { kind: 'expense' | 'income'; categories: ExpenseCategory[] | IncomeCategory[]; paymentMethods: PaymentMethod[]; saving: boolean; initialDate: string; onclose(): void; onsubmit(input: ExpenseInput | IncomeInput): Promise<void> }
  let { kind, categories, paymentMethods, saving, initialDate, onclose, onsubmit }: Props = $props();
  let date = $derived(initialDate); let amount = $state(''); let categoryId = $derived(categories[0]?.id ?? ''); let paymentMethodId = $derived(paymentMethods[0]?.id ?? ''); let description = $state('');
  function submit(event: SubmitEvent) { event.preventDefault(); return onsubmit(kind === 'expense' ? { transaction_date: date, amount, category_id: categoryId, payment_method_id: paymentMethodId, recurring_expense_id: null, description: description || null } : { transaction_date: date, amount, category_id: categoryId, description: description || null }); }
</script>

<div class="fixed inset-0 z-50 grid place-items-center bg-[#15241e]/55 p-4" role="presentation" onclick={(event) => { if (event.target === event.currentTarget) onclose(); }}>
  <div class="w-full max-w-lg rounded-2xl bg-[#fbfaf6] p-6 shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="transaction-title">
    <div class="flex items-center justify-between"><div><p class="text-xs font-bold tracking-widest text-[#39705d]">NEW ENTRY</p><h2 class="mt-1 font-serif text-2xl font-semibold" id="transaction-title">{kind === 'expense' ? '支出' : '収入'}を登録</h2></div><button class="grid size-9 place-items-center rounded-full hover:bg-[#eae7df]" aria-label="閉じる" onclick={onclose}>✕</button></div>
    <form class="mt-6 grid gap-4" onsubmit={submit}>
      <label class="grid gap-1.5 text-sm font-semibold">日付<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5" type="date" required bind:value={date} /></label>
      <label class="grid gap-1.5 text-sm font-semibold">金額<input class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5 text-lg" type="number" min="1" step="1" required bind:value={amount} /></label>
      <label class="grid gap-1.5 text-sm font-semibold">カテゴリ<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5" required bind:value={categoryId}>{#each categories as category (category.id)}<option value={category.id}>{category.name}</option>{/each}</select></label>
      {#if kind === 'expense'}<label class="grid gap-1.5 text-sm font-semibold">支払方法<select class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5" required bind:value={paymentMethodId}>{#each paymentMethods as method (method.id)}<option value={method.id}>{method.name}</option>{/each}</select></label>{/if}
      <label class="grid gap-1.5 text-sm font-semibold"><span>メモ <span class="font-normal text-[#838b86]">（任意）</span></span><textarea class="min-h-20 rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5" bind:value={description}></textarea></label>
      <div class="mt-2 flex justify-end gap-3"><button class="rounded-xl px-4 py-2.5 text-sm font-semibold" type="button" onclick={onclose}>キャンセル</button><button class="rounded-xl bg-[#245c4a] px-5 py-2.5 text-sm font-bold text-white disabled:opacity-50" type="submit" disabled={saving}>{saving ? '登録中…' : '登録する'}</button></div>
    </form>
  </div>
</div>
