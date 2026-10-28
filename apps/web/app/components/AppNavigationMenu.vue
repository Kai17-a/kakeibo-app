<script setup lang="ts">
import type { NavigationMenuItem } from "~/types/ui";
defineProps<{
  items: NavigationMenuItem[] | NavigationMenuItem[][];
  orientation?: string;
  highlight?: boolean;
}>();
</script>
<template>
  <nav :class="orientation === 'vertical' ? 'app-navigation-vertical' : 'app-navigation'">
    <NuxtLink
      v-for="(item, index) in items.flat()"
      :key="index"
      :to="item.to || '/'"
      :class="{ 'app-link-active': item.active }"
      @click="item.onSelect?.()"
      ><AppIcon v-if="item.icon" :name="item.icon" /><span>{{ item.label }}</span></NuxtLink
    >
  </nav>
</template>
