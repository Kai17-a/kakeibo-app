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
  const editing = $derived(Boolean(initial));

  function submit(event: SubmitEvent) {
    event.preventDefault();
    return onsubmit({
      url: url.trim(),
      description: description.trim() || null,
      is_active: active,
    });
  }
</script>

<Dialog.Root open onOpenChange={(open) => !open && onclose()}>
  <Dialog.Content class="max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>Webhook URLを{editing ? '編集' : '追加'}</Dialog.Title>
      <Dialog.Description>
        支出・収入の登録時に通知を送信するURLを{editing ? '更新' : '登録'}します。
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
            for="webhook-active">通知を有効にする</Field.FieldLabel
          >
        </Field.Field>
      </Field.FieldGroup>
      <Dialog.Footer>
        <Button variant="outline" type="button" onclick={onclose}>キャンセル</Button>
        <Button type="submit" disabled={saving || !url.trim()}>
          {#if saving}<Spinner data-icon="inline-start" />{editing
              ? '更新'
              : '登録'}中…{:else}Webhook URLを{editing ? '更新' : '追加'}{/if}
        </Button>
      </Dialog.Footer>
    </form>
  </Dialog.Content>
</Dialog.Root>
