import { CalendarDate, type DateValue } from "@internationalized/date";

function calendarDate(year: number, month: number, day: number): CalendarDate | undefined {
  if (year < 1) return undefined;

  try {
    const value = new CalendarDate(year, month, day);
    return value.year === year && value.month === month && value.day === day ? value : undefined;
  } catch {
    return undefined;
  }
}

export function dateStringToCalendarDate(value: string): CalendarDate | undefined {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return undefined;

  return calendarDate(Number(match[1]), Number(match[2]), Number(match[3]));
}

export function monthStringToCalendarDate(value: string): CalendarDate | undefined {
  const match = /^(\d{4})-(\d{2})$/.exec(value);
  if (!match) return undefined;

  return calendarDate(Number(match[1]), Number(match[2]), 1);
}

export function calendarDateToDateString(value: DateValue | undefined): string {
  if (!value) return "";
  return `${String(value.year).padStart(4, "0")}-${String(value.month).padStart(2, "0")}-${String(value.day).padStart(2, "0")}`;
}

export function calendarDateToMonthString(value: DateValue | undefined): string {
  if (!value) return "";
  return `${String(value.year).padStart(4, "0")}-${String(value.month).padStart(2, "0")}`;
}
