<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { WebhookUrl, WebhookUrlInput } from '$lib/types';

  interface Props {
    saving: boolean;
    initial?: WebhookUrl;
    onclose(): void;
    onsubmit(input: WebhookUrlInput): Promise<void>;
  }

  let { saving, initial, onclose, onsubmit }: Props = $props();
  let url = $state(untrack(() => initial?.url ?? ''));
  let description = $state(untrack(() => initial?.description ?? ''));
  let active = $state(untrack(() => initial?.is_active ?? true));
  let expenseCreated = $state(untrack(() => initial?.events.includes('expense.created') ?? true));
  let incomeCreated = $state(untrack(() => initial?.events.includes('income.created') ?? true));
  let budgetExceeded = $state(untrack(() => initial?.events.includes('budget.exceeded') ?? true));
  const events = $derived(
    [
      expenseCreated && 'expense.created',
      incomeCreated && 'income.created',
      budgetExceeded && 'budget.exceeded',
    ].filter((event): event is string => Boolean(event)),
  );
  const editing = $derived(Boolean(initial));

  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      url: url.trim(),
      description: description.trim() || null,
      is_active: active,
      events: [...events],
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>Webhook URLを{editing ? '編集' : '追加'}</Dialog.Title>
      <Dialog.Description>
        支出・収入の登録時や予算超過時に通知を送信するURLを{editing ? '更新' : '登録'}します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel for="webhook-url">Webhook URL</Field.FieldLabel>
          <Input
            id="webhook-url"
            type="url"
            required
            placeholder="https://example.com/webhook"
            bind:value={url}
          />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="webhook-description">説明（任意）</Field.FieldLabel>
          <Textarea
            id="webhook-description"
            maxlength={500}
            placeholder="通知先の用途やメモを入力"
            bind:value={description}
          />
        </Field.Field>
        <Field.Field orientation="horizontal">
          <Checkbox id="webhook-active" bind:checked={active} /><Field.FieldLabel
            for="webhook-active"
          >
            通知を有効にする
          </Field.FieldLabel>
        </Field.Field>
        <Field.FieldSet>
          <Field.FieldLegend>通知するイベント</Field.FieldLegend>
          <Field.FieldGroup>
            <Field.Field orientation="horizontal">
              <Checkbox id="webhook-event-expense" bind:checked={expenseCreated} />
              <Field.FieldLabel for="webhook-event-expense">支出登録</Field.FieldLabel>
            </Field.Field>
            <Field.Field orientation="horizontal">
              <Checkbox id="webhook-event-income" bind:checked={incomeCreated} />
              <Field.FieldLabel for="webhook-event-income">収入登録</Field.FieldLabel>
            </Field.Field>
            <Field.Field orientation="horizontal">
              <Checkbox id="webhook-event-budget" bind:checked={budgetExceeded} />
              <Field.FieldLabel for="webhook-event-budget">予算超過</Field.FieldLabel>
            </Field.Field>
          </Field.FieldGroup>
          {#if events.length === 0}
            <Field.FieldDescription>1つ以上のイベントを選択してください。</Field.FieldDescription>
          {/if}
        </Field.FieldSet>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving || !url.trim() || events.length === 0}>
          {#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}Webhook URLを{editing ? '更新' : '追加'}{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
