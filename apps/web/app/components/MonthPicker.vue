<script setup lang="ts">
import type { DateValue } from '@internationalized/date'
import { calendarDateToMonthString, monthStringToCalendarDate } from '~/utils/calendar-date'

const props = withDefaults(defineProps<{
  modelValue: string
  disabled?: boolean
  id?: string
  name?: string
  ariaLabel?: string
}>(), {
  id: undefined,
  name: undefined,
  ariaLabel: '対象月'
})

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const open = ref(false)
const { id, size, disabled } = useFormField(props)

const calendarValue = computed<DateValue | undefined>({
  get: () => monthStringToCalendarDate(props.modelValue),
  set: (value) => {
    if (!value) return
    emit('update:modelValue', calendarDateToMonthString(value))
    open.value = false
  }
})
const label = computed(() => {
  const value = calendarValue.value
  return value ? `${value.year}年${value.month}月` : '月を選択'
})
</script>

<template>
  <UPopover v-model:open="open">
    <UButton
      :id="id"
      :size="size"
      color="neutral"
      variant="outline"
      icon="i-lucide-calendar"
      :disabled="disabled"
      :aria-label="`${ariaLabel}: ${label}`"
      class="w-full justify-start font-normal"
    >
      {{ label }}
    </UButton>
    <template #content>
      <UCalendar
        v-model="calendarValue"
        type="month"
        class="p-2"
      />
    </template>
  </UPopover>
</template>
