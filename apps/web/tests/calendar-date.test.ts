import assert from 'node:assert/strict'
import { test } from 'node:test'
import { CalendarDate } from '@internationalized/date'
import { calendarDateToDateString, calendarDateToMonthString, dateStringToCalendarDate, monthStringToCalendarDate } from '../app/utils/calendar-date.ts'

test('converts valid date strings to CalendarDate values and back', () => {
  const value = dateStringToCalendarDate('2026-09-24')

  assert.deepEqual(value, new CalendarDate(2026, 9, 24))
  assert.equal(calendarDateToDateString(value), '2026-09-24')
})

test('converts valid month strings to first-of-month CalendarDate values and back', () => {
  const value = monthStringToCalendarDate('2026-09')

  assert.deepEqual(value, new CalendarDate(2026, 9, 1))
  assert.equal(calendarDateToMonthString(value), '2026-09')
})

test('returns undefined for empty or invalid date strings', () => {
  for (const value of ['', '2026-9-24', '2026-02-30', '0000-01-01', 'not-a-date']) {
    assert.equal(dateStringToCalendarDate(value), undefined)
  }
})

test('returns undefined for empty or invalid month strings', () => {
  for (const value of ['', '2026-9', '2026-00', '2026-13', '0000-01', 'not-a-month']) {
    assert.equal(monthStringToCalendarDate(value), undefined)
  }
})

test('returns an empty string when serializing no calendar value', () => {
  assert.equal(calendarDateToDateString(undefined), '')
  assert.equal(calendarDateToMonthString(undefined), '')
})
