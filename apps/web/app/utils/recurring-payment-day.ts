export const MONTH_END_PAYMENT_DAY = 31;

type PaymentDayState = { payment_day: string; is_month_end: boolean };

export function recurringPaymentDayForm(paymentDay = 1) {
  return {
    payment_day: String(paymentDay),
    is_month_end: paymentDay === MONTH_END_PAYMENT_DAY,
  };
}

export function setRecurringMonthEnd(state: PaymentDayState, enabled: boolean) {
  state.is_month_end = enabled;
  if (enabled) state.payment_day = String(MONTH_END_PAYMENT_DAY);
  else if (state.payment_day === String(MONTH_END_PAYMENT_DAY)) state.payment_day = "1";
}

export function updateRecurringPaymentDay(state: PaymentDayState, value: unknown) {
  state.payment_day = String(value ?? "");
  state.is_month_end = state.payment_day === String(MONTH_END_PAYMENT_DAY);
}

export function recurringPaymentDayValue(state: PaymentDayState) {
  return state.is_month_end ? MONTH_END_PAYMENT_DAY : Number(state.payment_day);
}

export function formatRecurringSchedule(paymentDay: number, spaced = false) {
  if (paymentDay === MONTH_END_PAYMENT_DAY) return "毎月末";
  const separator = spaced ? " " : "";
  return `毎月${separator}${paymentDay}${separator}日`;
}

export function formatRecurringPaymentDay(paymentDay: number | string) {
  return Number(paymentDay) === MONTH_END_PAYMENT_DAY ? "月末" : `${paymentDay}日`;
}
