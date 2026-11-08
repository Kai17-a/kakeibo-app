interface CrudCollectionOptions<T extends { id: string }, I> {
  /** The item name used in messages, e.g. 「支払方法」 for 「支払方法を追加しました」. */
  label: MaybeRefOrGetter<string>;
  /** Loads the items; may also load related data the page shows alongside them. */
  fetch: () => Promise<T[]>;
  create: (input: I) => Promise<T>;
  update: (id: string, input: I) => Promise<T>;
  remove: (id: string) => Promise<void>;
  /** Runs after a successful save, e.g. to refresh data derived from the items. */
  afterSave?: (items: T[]) => Promise<void>;
  /** The message shown when deleting fails; defaults to the API error message. */
  removeErrorMessage?: (error: unknown) => string;
}

/**
 * State and actions of a settings list: loading with retry, the add/edit form modal and the
 * delete confirmation. The page owns the form fields and builds the input passed to `save`.
 */
export function useCrudCollection<T extends { id: string }, I>(
  options: CrudCollectionOptions<T, I>,
) {
  const toast = useToast();
  const items = ref<T[]>([]) as Ref<T[]>;
  const loading = ref(true);
  const loadError = ref("");

  async function load() {
    loading.value = true;
    loadError.value = "";
    try {
      items.value = await options.fetch();
    } catch (error) {
      loadError.value = apiErrorMessage(error);
    } finally {
      loading.value = false;
    }
  }
  onMounted(load);

  // --- add / edit form ---
  const formOpen = ref(false);
  const editingId = ref<string | null>(null);
  const saving = ref(false);
  const formError = ref("");

  /** Opens the form to edit `item`, or to add a new item when omitted. */
  function openForm(item?: T) {
    editingId.value = item?.id ?? null;
    formError.value = "";
    formOpen.value = true;
  }

  async function save(input: I) {
    if (saving.value) return;
    saving.value = true;
    formError.value = "";
    try {
      const editing = editingId.value;
      const result = editing ? await options.update(editing, input) : await options.create(input);
      items.value = editing
        ? items.value.map((item) => (item.id === result.id ? result : item))
        : [...items.value, result];
      formOpen.value = false;
      toast.add({
        title: `${toValue(options.label)}を${editing ? "更新" : "追加"}しました`,
        color: "success",
      });
      await options.afterSave?.(items.value);
    } catch (error) {
      formError.value = apiErrorMessage(error);
    } finally {
      saving.value = false;
    }
  }

  // --- delete confirmation ---
  const deleteOpen = ref(false);
  const deleting = ref<T | null>(null) as Ref<T | null>;
  const removing = ref(false);
  const deleteError = ref("");

  function askDelete(item: T) {
    deleting.value = item;
    deleteError.value = "";
    deleteOpen.value = true;
  }

  async function remove() {
    if (!deleting.value || removing.value) return;
    removing.value = true;
    deleteError.value = "";
    const id = deleting.value.id;
    try {
      await options.remove(id);
      items.value = items.value.filter((item) => item.id !== id);
      deleteOpen.value = false;
      toast.add({ title: `${toValue(options.label)}を削除しました`, color: "success" });
    } catch (error) {
      deleteError.value = options.removeErrorMessage?.(error) ?? apiErrorMessage(error);
    } finally {
      removing.value = false;
    }
  }

  return {
    items,
    loading,
    loadError,
    load,
    formOpen,
    editingId,
    saving,
    formError,
    openForm,
    save,
    deleteOpen,
    deleting,
    removing,
    deleteError,
    askDelete,
    remove,
  };
}
