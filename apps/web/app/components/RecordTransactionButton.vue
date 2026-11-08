<script setup lang="ts">
const emit = defineEmits<{ click: [] }>();
</script>

<!-- Opens the transaction form. -->
<template>
  <UButton
    class="min-h-[44px] min-w-[44px] sm:min-h-0 sm:min-w-0"
    icon="i-lucide-plus"
    aria-label="記録する"
    @click="emit('click')"
  >
    <span>記録する</span>
  </UButton>
</template>
