<template>
  <section class="ui-card">
    <header v-if="$slots.header" class="ui-card-header"><slot name="header" /></header>
    <div class="ui-card-body"><slot /></div>
    <footer v-if="$slots.footer" class="ui-card-footer"><slot name="footer" /></footer>
  </section>
</template>
