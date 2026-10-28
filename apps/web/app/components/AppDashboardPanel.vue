<script setup lang="ts">
defineProps<{ ui?: { body?: string } }>();
</script>
<template>
  <main class="app-panel">
    <header><slot name="header" /></header>
    <div class="app-panel-body" :class="ui?.body"><slot name="body" /><slot /></div>
  </main>
</template>
