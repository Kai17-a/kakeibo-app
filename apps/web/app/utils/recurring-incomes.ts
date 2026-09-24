import type { RecurringIncome, RecurringIncomeInput } from "../types/settings";

export function recurringIncomeForm(item?: RecurringIncome, today = new Date()) {
  return {
    name: item?.name ?? "",
    amount: item?.amount ?? "",
    payment_day: String(item?.payment_day ?? 1),
    start_date:
      item?.start_date ??
      `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`,
    end_date: item?.end_date ?? "",
    category_id: item?.category_id ?? "",
    is_active: item?.is_active ?? true,
    is_variable: item?.is_variable ?? false,
    description: item?.description ?? "",
  };
}
type RecurringIncomeForm = ReturnType<typeof recurringIncomeForm>;

function validDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value.startsWith("0000")) return false;
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
}

export function validateRecurringIncome(state: RecurringIncomeForm, categories: { id: string }[]) {
  const errors: { name: string; message: string }[] = [];
  if (!state.name.trim()) errors.push({ name: "name", message: "名称を入力してください。" });
  if (
    (!state.is_variable || state.amount.trim() !== "") &&
    (!/^\d+$/.test(state.amount.trim()) ||
      !Number.isSafeInteger(Number(state.amount)) ||
      Number(state.amount) < 1)
  ) {
    errors.push({ name: "amount", message: "金額を1以上の整数で入力してください。" });
  }
  const day = Number(state.payment_day);
  if (!Number.isInteger(day) || day < 1 || day > 31)
    errors.push({ name: "payment_day", message: "入金日を1〜31の整数で入力してください。" });
  if (!categories.some((item) => item.id === state.category_id))
    errors.push({ name: "category_id", message: "収入カテゴリを選択してください。" });
  if (!validDate(state.start_date))
    errors.push({ name: "start_date", message: "有効な開始日を入力してください。" });
  if (state.end_date && (!validDate(state.end_date) || state.end_date < state.start_date)) {
    errors.push({
      name: "end_date",
      message: "終了日は開始日以降の有効な日付を入力してください。",
    });
  }
  return errors;
}

export function recurringIncomeInput(state: RecurringIncomeForm): RecurringIncomeInput {
  return {
    ...state,
    name: state.name.trim(),
    amount: state.amount.trim() || null,
    payment_day: Number(state.payment_day),
    end_date: state.end_date || null,
    description: state.description.trim() || null,
  };
}
