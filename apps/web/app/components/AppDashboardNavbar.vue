<script setup lang="ts">
defineProps<{ title: string }>();
</script>
<template>
  <div class="app-navbar">
    <slot name="leading" />
    <h1 class="type-heading">{{ title }}</h1>
    <div class="ml-auto flex flex-wrap items-center gap-2"><slot name="right" /></div>
  </div>
</template>
