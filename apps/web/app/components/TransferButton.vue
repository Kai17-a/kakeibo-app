<script setup lang="ts">
const emit = defineEmits<{ click: [] }>();
</script>

<!-- Opens the transfer form. -->
<template>
  <UButton
    class="min-h-[44px] min-w-[44px] sm:min-h-0 sm:min-w-0"
    icon="i-lucide-arrow-right-left"
    color="neutral"
    variant="outline"
    aria-label="振替"
    @click="emit('click')"
  >
    <span>振替</span>
  </UButton>
</template>
