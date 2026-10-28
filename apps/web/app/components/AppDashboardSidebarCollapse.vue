<script setup lang="ts">
const open = useState("sidebar-open", () => false);
</script>
<template>
  <button
    class="btn btn-ghost-secondary"
    type="button"
    aria-label="サイドバーを切り替え"
    :aria-expanded="open"
    aria-controls="app-sidebar"
    @click="open = !open"
  >
    ☰
  </button>
</template>
