<script setup lang="ts">
import { icons } from "@iconify-json/lucide/icons.json";
const props = defineProps<{ name: string }>();
const icon = computed(() => icons[props.name.replace("i-lucide-", "") as keyof typeof icons]);
</script>
<template>
  <svg
    v-if="icon"
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="`0 0 24 24`"
    class="app-icon"
    aria-hidden="true"
    v-html="icon.body"
  />
</template>
