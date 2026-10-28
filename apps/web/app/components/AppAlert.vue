<script setup lang="ts">
withDefaults(
  defineProps<{
    title?: string;
    description?: string;
    color?: string;
    icon?: string;
    actions?: {
      label: string;
      onClick?: (event: MouseEvent) => void;
      color?: string;
      variant?: string;
      size?: string;
      icon?: string;
      loading?: boolean;
      disabled?: boolean;
    }[];
  }>(),
  { color: "info" },
);
</script>
<template>
  <div
    class="alert"
    :class="`alert-${color === 'error' ? 'danger' : color === 'neutral' ? 'secondary' : color}`"
    :role="color === 'error' ? 'alert' : 'status'"
  >
    <AppIcon v-if="icon" :name="icon" />
    <div>
      <strong v-if="title">{{ title }}</strong>
      <p v-if="description">{{ description }}</p>
      <slot />
      <div v-if="actions?.length" class="mt-3 flex flex-wrap gap-2">
        <AppButton v-for="(action, index) in actions" :key="index" v-bind="action">
          {{ action.label }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
