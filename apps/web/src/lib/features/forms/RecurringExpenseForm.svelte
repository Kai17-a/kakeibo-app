<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import * as NativeSelect from '$lib/components/ui/native-select';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
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
      amount: String(amount),
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

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto overscroll-contain sm:max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>固定費を登録</Dialog.Title>
      <Dialog.Description>毎月発生する家賃や通信費などの支出予定を登録します。</Dialog.Description>
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
          ><Field.FieldLabel for="recurring-amount">金額</Field.FieldLabel><Input
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
          ><Checkbox id="recurring-active" bind:checked={active} /><Field.FieldLabel
            for="recurring-active">登録後すぐに有効にする</Field.FieldLabel
          ></Field.Field
        >
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving}
          >{#if saving}<Spinner data-icon="inline-start" />登録中…{:else}固定費を登録{/if}</Button
        >
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
