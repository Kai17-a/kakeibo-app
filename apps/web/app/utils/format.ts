export const formatCurrency = (value: string | number) =>
  new Intl.NumberFormat("ja-JP", {
    style: "currency",
    currency: "JPY",
    maximumFractionDigits: 0,
  }).format(Number(value));

export const formatSignedCurrency = (
  value: string | number,
  direction: "auto" | "positive" | "negative" = "auto",
) => {
  const amount = Number(value);
  if (amount === 0) return formatCurrency(0);

  const sign =
    direction === "auto" ? (amount > 0 ? "+" : "−") : direction === "positive" ? "+" : "−";
  return `${sign}${formatCurrency(Math.abs(amount))}`;
};

export const formatDate = (value: string) =>
  new Intl.DateTimeFormat("ja-JP", {
    month: "short",
    day: "numeric",
    weekday: "short",
  }).format(new Date(`${value}T00:00:00`));

export const currentDate = () => {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${month}-${day}`;
};

export const currentMonth = () => currentDate().slice(0, 7);

export const isValidMonth = (value: string) => /^\d{4}-(0[1-9]|1[0-2])$/.test(value);

export const shiftMonth = (month: string, delta: number) => {
  const [year, monthNumber] = month.split("-").map(Number) as [number, number];
  const totalMonths = year * 12 + monthNumber - 1 + delta;
  const shiftedYear = Math.floor(totalMonths / 12);
  const shiftedMonth = (((totalMonths % 12) + 12) % 12) + 1;

  return `${String(shiftedYear).padStart(4, "0")}-${String(shiftedMonth).padStart(2, "0")}`;
};
