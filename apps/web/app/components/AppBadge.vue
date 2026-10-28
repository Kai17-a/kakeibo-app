<script setup lang="ts">
defineProps<{ color?: string; variant?: string }>();
</script>
<template>
  <span class="badge app-badge" :class="`text-${color === 'error' ? 'error' : color || 'muted'}`"
    ><slot
  /></span>
</template>
