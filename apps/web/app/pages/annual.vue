<script setup lang="ts">
// The annual summary moved to "/"; keep old links such as /annual?year=2026 working.
definePageMeta({ redirect: (to) => ({ path: "/", query: to.query }) });
</script>

<template>
  <div />
</template>
