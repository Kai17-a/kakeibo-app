<script setup lang="ts">
const emit = defineEmits<{ click: [] }>();
</script>

<!-- Opens the transaction form; the label collapses to the icon on narrow screens. -->
<template>
  <AppButton icon="i-lucide-plus" aria-label="記録する" @click="emit('click')">
    <span class="hidden sm:inline">記録する</span>
  </AppButton>
</template>
