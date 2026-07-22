<script lang="ts">
  import type { ExpenseCategory, PaymentMethod, RecurringExpenseInput } from '../../types';
  interface Props {
    categories: ExpenseCategory[];
    paymentMethods: PaymentMethod[];
    saving: boolean;
    onclose(): void;
    onsubmit(input: RecurringExpenseInput): Promise<void>;
  }
  let { categories, paymentMethods, saving, onclose, onsubmit }: Props = $props();
  let name = $state('');
  let amount = $state('');
  let paymentDay = $state(1);
  let startDate = $state(new Date().toISOString().slice(0, 10));
  let endDate = $state('');
  let categoryId = $derived(categories[0]?.id ?? '');
  let paymentMethodId = $derived(paymentMethods[0]?.id ?? '');
  let description = $state('');
  let active = $state(true);
  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      name,
      amount,
      payment_day: paymentDay,
      start_date: startDate,
      end_date: endDate || null,
      category_id: categoryId,
      payment_method_id: paymentMethodId,
      is_active: active,
      description: description || null,
    });
  }
</script>

<div
  class="fixed inset-0 z-50 grid place-items-center overflow-y-auto bg-[#15241e]/55 p-4"
  role="presentation"
  onclick={(event) => {
    if (event.target === event.currentTarget) onclose();
  }}
>
  <div
    class="my-auto w-full max-w-2xl rounded-2xl bg-[#fbfaf6] p-6 shadow-2xl"
    role="dialog"
    aria-modal="true"
    aria-labelledby="recurring-title"
  >
    <div class="flex items-center justify-between">
      <div>
        <p class="text-xs font-bold tracking-widest text-[#39705d]">FIXED EXPENSE</p>
        <h2 class="mt-1 font-serif text-2xl font-semibold" id="recurring-title">固定費を登録</h2>
      </div>
      <button
        class="grid size-9 place-items-center rounded-full hover:bg-[#eae7df]"
        aria-label="閉じる"
        onclick={onclose}>✕</button
      >
    </div>
    <p class="mt-2 text-sm text-[#717b75]">毎月発生する家賃や通信費などの支出予定を登録します。</p>
    <form class="mt-6 grid gap-4 sm:grid-cols-2" onsubmit={submit}>
      <label class="grid gap-1.5 text-sm font-semibold sm:col-span-2"
        >名称<input
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          required
          bind:value={name}
        /></label
      >
      <label class="grid gap-1.5 text-sm font-semibold"
        >金額<input
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          type="number"
          min="1"
          required
          bind:value={amount}
        /></label
      ><label class="grid gap-1.5 text-sm font-semibold"
        >毎月の支払日<input
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          type="number"
          min="1"
          max="31"
          required
          bind:value={paymentDay}
        /></label
      >
      <label class="grid gap-1.5 text-sm font-semibold"
        >カテゴリ<select
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          required
          bind:value={categoryId}
          >{#each categories as category (category.id)}<option value={category.id}
              >{category.name}</option
            >{/each}</select
        ></label
      ><label class="grid gap-1.5 text-sm font-semibold"
        >支払方法<select
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          required
          bind:value={paymentMethodId}
          >{#each paymentMethods as method (method.id)}<option value={method.id}
              >{method.name}</option
            >{/each}</select
        ></label
      >
      <label class="grid gap-1.5 text-sm font-semibold"
        >開始日<input
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          type="date"
          required
          bind:value={startDate}
        /></label
      ><label class="grid gap-1.5 text-sm font-semibold"
        ><span>終了日 <span class="font-normal text-[#838b86]">（任意）</span></span><input
          class="rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          type="date"
          min={startDate}
          bind:value={endDate}
        /></label
      >
      <label class="grid gap-1.5 text-sm font-semibold sm:col-span-2"
        ><span>備考 <span class="font-normal text-[#838b86]">（任意）</span></span><textarea
          class="min-h-20 rounded-xl border border-[#cbc9c0] bg-white px-3 py-2.5"
          bind:value={description}></textarea></label
      ><label class="flex items-center gap-2 text-sm font-semibold sm:col-span-2"
        ><input
          class="size-4 accent-[#245c4a]"
          type="checkbox"
          bind:checked={active}
        />登録後すぐに有効にする</label
      >
      <div class="mt-2 flex justify-end gap-3 sm:col-span-2">
        <button class="rounded-xl px-4 py-2.5 text-sm font-semibold" type="button" onclick={onclose}
          >キャンセル</button
        ><button
          class="rounded-xl bg-[#245c4a] px-5 py-2.5 text-sm font-bold text-white disabled:opacity-50"
          type="submit"
          disabled={saving}>{saving ? '登録中…' : '固定費を登録'}</button
        >
      </div>
    </form>
  </div>
</div>
