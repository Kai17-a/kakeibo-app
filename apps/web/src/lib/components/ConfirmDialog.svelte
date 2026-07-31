<script lang="ts">
  import * as AlertDialog from '$lib/components/ui/alert-dialog';

  interface Props {
    open: boolean;
    title: string;
    description: string;
    actionLabel?: string;
    onconfirm(): void;
    oncancel(): void;
  }

  let { open, title, description, actionLabel = '削除', onconfirm, oncancel }: Props = $props();
</script>

<AlertDialog.Root {open} onOpenChange={(value) => !value && oncancel()}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>{title}</AlertDialog.Title>
      <AlertDialog.Description>{description}</AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={oncancel}>キャンセル</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={onconfirm}
        >{actionLabel}</AlertDialog.Action
      >
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
