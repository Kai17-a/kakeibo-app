import type { WebhookUrlInput } from "../types/webhook";

export const webhookEvents = [
  { value: "expense.created", label: "支出登録" },
  { value: "income.created", label: "収入登録" },
  { value: "budget.exceeded", label: "予算超過" },
];

export function validateWebhook(state: Partial<WebhookUrlInput>) {
  const errors: { name: string; message: string }[] = [];
  const url = state.url?.trim() ?? "";
  try {
    if (!/^https?:\/\//.test(url) || !new URL(url).hostname) throw new Error("Invalid URL");
  } catch {
    errors.push({
      name: "url",
      message: "http:// または https:// で始まる有効なURLを入力してください。",
    });
  }
  if (!state.events?.length)
    errors.push({ name: "events", message: "1つ以上のイベントを選択してください。" });
  if ((state.description?.length ?? 0) > 500)
    errors.push({ name: "description", message: "説明は500文字以内で入力してください。" });
  return errors;
}
