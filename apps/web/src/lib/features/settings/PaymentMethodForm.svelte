<script lang="ts">
  import { untrack } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import * as Field from '$lib/components/ui/field';
  import { Input } from '$lib/components/ui/input';
  import { Spinner } from '$lib/components/ui/spinner';
  import { Textarea } from '$lib/components/ui/textarea';
  import type { PaymentMethod, PaymentMethodInput } from '$lib/types';

  interface Props {
    saving: boolean;
    initial?: PaymentMethod;
    onclose(): void;
    onsubmit(input: PaymentMethodInput): Promise<void>;
  }

  let { saving, initial, onclose, onsubmit }: Props = $props();
  let name = $state(untrack(() => initial?.name ?? ''));
  let description = $state(untrack(() => initial?.description ?? ''));
  let initialBalance = $state(untrack(() => initial?.initial_balance ?? ''));
  const editing = $derived(Boolean(initial));

  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      name: name.trim(),
      description: description.trim() || null,
      initial_balance: String(initialBalance).trim() || undefined,
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>支払方法を{editing ? '編集' : '追加'}</Dialog.Title>
      <Dialog.Description>
        支出の登録時に選択する支払方法を{editing ? '更新' : '作成'}します。
      </Dialog.Description>
    </Dialog.Header>
    <form class="flex flex-col gap-6" onsubmit={submit}>
      <Field.FieldGroup>
        <Field.Field>
          <Field.FieldLabel for="payment-method-initial-balance">初期残高（任意）</Field.FieldLabel>
          <Input
            id="payment-method-initial-balance"
            type="number"
            min="0"
            step="1"
            placeholder="例：100000"
            bind:value={initialBalance}
          />
          <Field.FieldDescription>
            残高管理を行う場合の開始時点の金額を入力してください。
          </Field.FieldDescription>
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="payment-method-name">支払方法名</Field.FieldLabel>
          <Input
            id="payment-method-name"
            required
            maxlength={100}
            placeholder="例：クレジットカード"
            bind:value={name}
          />
        </Field.Field>
        <Field.Field>
          <Field.FieldLabel for="payment-method-description">説明（任意）</Field.FieldLabel>
          <Textarea
            id="payment-method-description"
            maxlength={500}
            placeholder="支払方法の用途や対象を入力"
            bind:value={description}
          />
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving || !name.trim()}>
          {#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}支払方法を{editing ? '更新' : '追加'}{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
