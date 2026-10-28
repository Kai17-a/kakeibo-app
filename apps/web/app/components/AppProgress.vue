<script setup lang="ts">
withDefaults(defineProps<{ modelValue?: number; max?: number; color?: string; size?: string }>(), {
  modelValue: 0,
  max: 100,
});
</script>
<template>
  <progress
    class="app-progress"
    :max="max"
    :value="Math.max(0, Math.min(max, modelValue))"
    :style="{ accentColor: `var(--bs-${color === 'error' ? 'danger' : color || 'primary'})` }"
    :aria-label="`進捗 ${Math.round(modelValue)} / ${max}`"
  />
</template>
