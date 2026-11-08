import type { Transfer, TransferInput } from "~/types/transactions";
import { defaultTransactionDate } from "~/utils/transaction-date";

interface TransferEditorOptions {
  month: MaybeRefOrGetter<string>;
  transfers: Ref<Transfer[]>;
}

export function useTransferEditor({ month, transfers }: TransferEditorOptions) {
  const transactionsApi = useTransactionsApi();
  const toast = useToast();
  const transferFormOpen = ref(false);
  const editingTransfer = ref<Transfer | null>(null);
  const transferSaving = ref(false);
  const transferInitialDate = computed(() => defaultTransactionDate(toValue(month)));

  function openNewTransfer() {
    editingTransfer.value = null;
    transferFormOpen.value = true;
  }

  function editTransfer(item: Transfer) {
    editingTransfer.value = item;
    transferFormOpen.value = true;
  }

  function closeTransferForm() {
    transferFormOpen.value = false;
    editingTransfer.value = null;
  }

  async function saveTransfer(input: TransferInput, keepOpen: boolean): Promise<boolean> {
    if (transferSaving.value) return false;
    transferSaving.value = true;
    try {
      const editing = editingTransfer.value;
      if (editing) {
        const updated = await transactionsApi.updateTransfer(editing.id, input);
        transfers.value = transfers.value.map((item) => (item.id === updated.id ? updated : item));
        toast.add({ title: "振替を更新しました", color: "success" });
      } else {
        transfers.value = [await transactionsApi.createTransfer(input), ...transfers.value];
        toast.add({ title: "振替を登録しました", color: "success" });
      }
      if (!keepOpen) closeTransferForm();
      return true;
    } catch (error) {
      toast.add({ title: apiErrorMessage(error), color: "error" });
      return false;
    } finally {
      transferSaving.value = false;
    }
  }

  return {
    transferFormOpen,
    editingTransfer,
    transferSaving,
    transferInitialDate,
    openNewTransfer,
    editTransfer,
    closeTransferForm,
    saveTransfer,
  };
}
