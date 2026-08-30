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
  import { currentDate } from '../../format';
  import type { IncomeCategory, RecurringIncome, RecurringIncomeInput } from '../../types';

  interface Props {
    categories: IncomeCategory[];
    saving: boolean;
    initial?: RecurringIncome;
    onclose(): void;
    onsubmit(input: RecurringIncomeInput): Promise<void>;
  }
  let { categories, saving, initial, onclose, onsubmit }: Props = $props();
  let name = $state(untrack(() => initial?.name ?? ''));
  let amount = $state(untrack(() => initial?.amount ?? ''));
  let paymentDay = $state(untrack(() => initial?.payment_day ?? 1));
  let startDate = $state(untrack(() => initial?.start_date ?? currentDate()));
  let endDate = $state(untrack(() => initial?.end_date ?? ''));
  let categoryId = $derived(initial?.category_id ?? categories[0]?.id ?? '');
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
      is_active: active,
      is_variable: variable,
      description: description || null,
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto overscroll-contain sm:max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>定期収入を{editing ? '編集' : '登録'}</Dialog.Title>
      <Dialog.Description>
        毎月発生する給与などの収入予定を{editing ? '更新' : '登録'}します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup class="grid sm:grid-cols-2">
        <Field.Field class="sm:col-span-2">
          <Field.FieldLabel for="recurring-income-name">名称</Field.FieldLabel>
          <Input id="recurring-income-name" required bind:value={name} />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="recurring-income-amount">
            {variable ? '金額（目安）' : '金額'}
          </Field.FieldLabel>
          <Input id="recurring-income-amount" type="number" min="1" required bind:value={amount} />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="recurring-income-day">毎月の入金日</Field.FieldLabel>
          <Input
            id="recurring-income-day"
            type="number"
            min="1"
            max="31"
            required
            bind:value={paymentDay}
          />
        </Field.Field>
        <Field.Field class="sm:col-span-2">
          <Field.FieldLabel for="recurring-income-category">カテゴリ</Field.FieldLabel>
          <NativeSelect.Root id="recurring-income-category" required bind:value={categoryId}>
            {#each categories as category (category.id)}
              <NativeSelect.Option value={category.id}>{category.name}</NativeSelect.Option>
            {/each}
          </NativeSelect.Root>
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="recurring-income-start">開始日</Field.FieldLabel>
          <Input id="recurring-income-start" type="date" required bind:value={startDate} />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="recurring-income-end">終了日（任意）</Field.FieldLabel>
          <Input id="recurring-income-end" type="date" min={startDate} bind:value={endDate} />
        </Field.Field>
        <Field.Field class="sm:col-span-2">
          <Field.FieldLabel for="recurring-income-description">備考（任意）</Field.FieldLabel>
          <Textarea id="recurring-income-description" bind:value={description} />
        </Field.Field>
        <Field.Field orientation="horizontal" class="sm:col-span-2">
          <Checkbox id="recurring-income-variable" bind:checked={variable} />
          <Field.FieldLabel for="recurring-income-variable">
            金額が月ごとに変動する（準固定収入）
          </Field.FieldLabel>
        </Field.Field>
        <Field.Field orientation="horizontal" class="sm:col-span-2">
          <Checkbox id="recurring-income-active" bind:checked={active} />
          <Field.FieldLabel for="recurring-income-active">
            {editing ? '有効にする' : '登録後すぐに有効にする'}
          </Field.FieldLabel>
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving}>
          {#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}定期収入を{editing ? '更新' : '登録'}{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
