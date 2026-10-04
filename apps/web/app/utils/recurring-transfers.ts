import type { PaymentMethod, RecurringTransfer, RecurringTransferInput } from "../types/settings";
import { recurringPaymentDayForm, recurringPaymentDayValue } from "./recurring-payment-day.ts";

export function recurringTransferForm(item?: RecurringTransfer, today = new Date()) {
  return {
    name: item?.name ?? "",
    amount: item?.amount ?? "",
    ...recurringPaymentDayForm(item?.payment_day),
    start_date:
      item?.start_date ??
      `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`,
    end_date: item?.end_date ?? "",
    from_payment_method_id: item?.from_payment_method_id ?? "",
    to_payment_method_id: item?.to_payment_method_id ?? "",
    is_active: item?.is_active ?? true,
    description: item?.description ?? "",
  };
}
type RecurringTransferForm = ReturnType<typeof recurringTransferForm>;

function validDate(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || value.startsWith("0000")) return false;
  const date = new Date(`${value}T00:00:00Z`);
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
}

export function validateRecurringTransfer(state: RecurringTransferForm, methods: PaymentMethod[]) {
  const errors: { name: string; message: string }[] = [];
  if (!state.name.trim()) errors.push({ name: "name", message: "名称を入力してください。" });
  if (
    !/^\d+$/.test(state.amount.trim()) ||
    !Number.isSafeInteger(Number(state.amount)) ||
    Number(state.amount) < 1
  )
    errors.push({ name: "amount", message: "金額を1以上の整数で入力してください。" });
  const day = Number(state.payment_day);
  if (!Number.isInteger(day) || day < 1 || day > 31)
    errors.push({ name: "payment_day", message: "振替日を1〜31の整数で入力してください。" });
  if (!methods.some((item) => item.id === state.from_payment_method_id))
    errors.push({ name: "from_payment_method_id", message: "移動元を選択してください。" });
  if (!methods.some((item) => item.id === state.to_payment_method_id))
    errors.push({ name: "to_payment_method_id", message: "移動先を選択してください。" });
  if (state.from_payment_method_id && state.from_payment_method_id === state.to_payment_method_id)
    errors.push({
      name: "to_payment_method_id",
      message: "移動元と異なる支払方法を選択してください。",
    });
  if (!validDate(state.start_date))
    errors.push({ name: "start_date", message: "有効な開始日を入力してください。" });
  if (state.end_date && (!validDate(state.end_date) || state.end_date < state.start_date))
    errors.push({
      name: "end_date",
      message: "終了日は開始日以降の有効な日付を入力してください。",
    });
  return errors;
}

export function recurringTransferInput(state: RecurringTransferForm): RecurringTransferInput {
  return {
    name: state.name.trim(),
    amount: state.amount.trim(),
    payment_day: recurringPaymentDayValue(state),
    start_date: state.start_date,
    end_date: state.end_date || null,
    from_payment_method_id: state.from_payment_method_id,
    to_payment_method_id: state.to_payment_method_id,
    is_active: state.is_active,
    description: state.description.trim() || null,
  };
}
