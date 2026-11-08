<script setup lang="ts">
import type { DateValue } from "@internationalized/date";
import { calendarDateToDateString, dateStringToCalendarDate } from "~/utils/calendar-date";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    min?: string;
    placeholder?: string;
    clearable?: boolean;
    disabled?: boolean;
    required?: boolean;
    id?: string;
    name?: string;
  }>(),
  {
    min: undefined,
    placeholder: "日付を選択",
    id: undefined,
    name: undefined,
  },
);

const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const open = ref(false);
const { id, size, color, disabled, ariaAttrs, emitFormInput, emitFormChange } = useFormField(props);

const calendarValue = computed<DateValue | undefined>({
  get: () => dateStringToCalendarDate(props.modelValue),
  set: (value) => {
    emit("update:modelValue", calendarDateToDateString(value));
    emitFormInput();
    emitFormChange();
    if (value) open.value = false;
  },
});
const minValue = computed(() => (props.min ? dateStringToCalendarDate(props.min) : undefined));
const label = computed(() => {
  const value = calendarValue.value;
  return value ? `${value.year}年${value.month}月${value.day}日` : props.placeholder;
});

function clear() {
  emit("update:modelValue", "");
  emitFormInput();
  emitFormChange();
}
</script>

<template>
  <div class="flex w-full">
    <UPopover v-model:open="open" class="min-w-0 flex-1">
      <UButton
        :id="id"
        :size="size"
        :color="color || 'neutral'"
        variant="outline"
        icon="i-lucide-calendar"
        :disabled="disabled"
        :aria-required="required || undefined"
        v-bind="ariaAttrs"
        class="w-full justify-start font-normal"
        :class="clearable && modelValue ? 'rounded-e-none' : undefined"
      >
        <span :class="modelValue ? 'text-default' : 'text-dimmed'">
          {{ label }}
        </span>
      </UButton>
      <template #content>
        <UCalendar v-model="calendarValue" :min-value="minValue" class="p-2" />
      </template>
    </UPopover>
    <UButton
      v-if="clearable && modelValue"
      icon="i-lucide-x"
      color="neutral"
      variant="outline"
      :size="size"
      :disabled="disabled"
      aria-label="日付をクリア"
      class="-ms-px shrink-0 rounded-s-none"
      @click="clear"
    />
  </div>
</template>
