<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
  import type {
    ExpenseCategory,
    PaymentMethod,
    RecurringExpense,
    RecurringExpenseInput,
  } from '../../types';
  interface Props {
    categories: ExpenseCategory[];
    paymentMethods: PaymentMethod[];
    saving: boolean;
    initial?: RecurringExpense;
    onclose(): void;
    onsubmit(input: RecurringExpenseInput): Promise<void>;
  }
  let { categories, paymentMethods, saving, initial, onclose, onsubmit }: Props = $props();
  let name = $state(untrack(() => initial?.name ?? ''));
  let amount = $state(untrack(() => initial?.amount ?? ''));
  let paymentDay = $state(untrack(() => initial?.payment_day ?? 1));
  let startDate = $state(
    untrack(() => initial?.start_date ?? new Date().toISOString().slice(0, 10)),
  );
  let endDate = $state(untrack(() => initial?.end_date ?? ''));
  let categoryId = $derived(initial?.category_id ?? categories[0]?.id ?? '');
  let paymentMethodId = $derived(initial?.payment_method_id ?? paymentMethods[0]?.id ?? '');
  let description = $state(untrack(() => initial?.description ?? ''));
  let active = $state(untrack(() => initial?.is_active ?? true));
  let variable = $state(untrack(() => initial?.is_variable ?? false));
  const editing = $derived(Boolean(initial));
  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      name,
      amount: String(amount),
      payment_day: paymentDay,
      start_date: startDate,
      end_date: endDate || null,
      category_id: categoryId,
      payment_method_id: paymentMethodId,
      is_active: active,
      is_variable: variable,
      description: description || null,
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto overscroll-contain sm:max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>固定費を{editing ? '編集' : '登録'}</Dialog.Title>
      <Dialog.Description>
        毎月発生する家賃や通信費などの支出予定を{editing ? '更新' : '登録'}します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup class="grid sm:grid-cols-2">
        <Field.Field class="sm:col-span-2"
          ><Field.FieldLabel for="recurring-name">名称</Field.FieldLabel><Input
            id="recurring-name"
            required
            bind:value={name}
          /></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-amount"
            >{variable ? '金額（目安）' : '金額'}</Field.FieldLabel
          ><Input
            id="recurring-amount"
            type="number"
            min="1"
            required
            bind:value={amount}
          /></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-day">毎月の支払日</Field.FieldLabel><Input
            id="recurring-day"
            type="number"
            min="1"
            max="31"
            required
            bind:value={paymentDay}
          /></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-category">カテゴリ</Field.FieldLabel><NativeSelect.Root
            id="recurring-category"
            required
            bind:value={categoryId}
            >{#each categories as category (category.id)}<NativeSelect.Option value={category.id}
                >{category.name}</NativeSelect.Option
              >{/each}</NativeSelect.Root
          ></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-payment">支払方法</Field.FieldLabel><NativeSelect.Root
            id="recurring-payment"
            required
            bind:value={paymentMethodId}
            >{#each paymentMethods as method (method.id)}<NativeSelect.Option value={method.id}
                >{method.name}</NativeSelect.Option
              >{/each}</NativeSelect.Root
          ></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-start">開始日</Field.FieldLabel><Input
            id="recurring-start"
            type="date"
            required
            bind:value={startDate}
          /></Field.Field
        >
        <Field.Field
          ><Field.FieldLabel for="recurring-end">終了日（任意）</Field.FieldLabel><Input
            id="recurring-end"
            type="date"
            min={startDate}
            bind:value={endDate}
          /></Field.Field
        >
        <Field.Field class="sm:col-span-2"
          ><Field.FieldLabel for="recurring-description">備考（任意）</Field.FieldLabel><Textarea
            id="recurring-description"
            bind:value={description}
          /></Field.Field
        >
        <Field.Field orientation="horizontal" class="sm:col-span-2"
          ><Checkbox id="recurring-variable" bind:checked={variable} /><Field.FieldLabel
            for="recurring-variable">金額が月ごとに変動する（準固定費）</Field.FieldLabel
          ></Field.Field
        >
        <Field.Field orientation="horizontal" class="sm:col-span-2"
          ><Checkbox id="recurring-active" bind:checked={active} /><Field.FieldLabel
            for="recurring-active"
            >{editing ? '有効にする' : '登録後すぐに有効にする'}</Field.FieldLabel
          ></Field.Field
        >
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving}
          >{#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}固定費を{editing ? '更新' : '登録'}{/if}</Button
        >
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
