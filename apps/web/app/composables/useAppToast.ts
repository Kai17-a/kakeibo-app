interface AppToast {
  id: number;
  title: string;
  description?: string;
  color?: string;
  duration?: number;
}
export function useAppToast() {
  const toasts = useState<AppToast[]>("app-toasts", () => []);
  const nextId = useState("app-toast-id", () => 0);
  function remove(id: number) {
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
  }
  function add(input: Omit<AppToast, "id">) {
    const id = ++nextId.value;
    toasts.value.push({ ...input, id });
    const duration = input.duration ?? (input.color === "error" ? 0 : 6000);
    if (duration > 0) setTimeout(() => remove(id), duration);
    return id;
  }
  return { toasts, add, remove };
}
