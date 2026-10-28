<script setup lang="ts">
import type { RouteLocationRaw } from "vue-router";
withDefaults(
  defineProps<{
    to?: RouteLocationRaw;
    type?: "button" | "submit" | "reset";
    color?: string;
    variant?: string;
    size?: string;
    icon?: string;
    loading?: boolean;
    disabled?: boolean;
  }>(),
  { type: "button", color: "primary", variant: "solid" },
);
const fill = (variant: string) =>
  variant === "solid" ? "" : variant === "link" ? "ghost-" : `${variant}-`;
const tone = (color: string) =>
  color === "error" ? "danger" : color === "neutral" ? "secondary" : color;
</script>
<template>
  <NuxtLink
    v-if="to && !disabled && !loading"
    :to="to"
    class="btn"
    :class="[`btn-${fill(variant)}${tone(color)}`, { 'btn-sm': size === 'sm' || size === 'xs' }]"
    ><AppIcon v-if="icon" :name="icon" /><slot
  /></NuxtLink>
  <button
    v-else
    :type="type"
    class="btn"
    :class="[`btn-${fill(variant)}${tone(color)}`, { 'btn-sm': size === 'sm' || size === 'xs' }]"
    :disabled="disabled || loading"
    :aria-busy="loading || undefined"
  >
    <span v-if="loading" aria-hidden="true">…</span
    ><AppIcon v-else-if="icon" :name="icon" /><slot />
  </button>
</template>
