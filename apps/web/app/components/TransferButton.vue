<script setup lang="ts">
const emit = defineEmits<{ click: [] }>();
</script>

<!-- Opens the transfer form; the label collapses to the icon on narrow screens. -->
<template>
  <UButton
    icon="i-lucide-arrow-right-left"
    color="neutral"
    variant="outline"
    aria-label="振替"
    @click="emit('click')"
  >
    <span class="hidden sm:inline">振替</span>
  </UButton>
</template>
