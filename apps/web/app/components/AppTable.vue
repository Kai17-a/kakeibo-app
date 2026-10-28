<script setup lang="ts" generic="T extends object">
import type { TableColumn } from "~/types/ui";
defineProps<{
  data: T[];
  columns: TableColumn<T>[];
  ui?: { root?: string; base?: string; th?: string; td?: string };
  loading?: boolean;
}>();
function value(row: T, column: TableColumn<T>) {
  return column.cell ? column.cell({ row: { original: row } }) : row[column.accessorKey as keyof T];
}
</script>
<template>
  <table class="table" :class="[ui?.root, ui?.base]">
    <thead>
      <tr>
        <th
          v-for="column in columns"
          :key="column.id || column.accessorKey"
          scope="col"
          :class="[ui?.th, column.meta?.class?.th]"
        >
          {{ column.header }}
        </th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(original, index) in data" :key="String('id' in original ? original.id : index)">
        <td
          v-for="column in columns"
          :key="column.id || column.accessorKey"
          :class="[ui?.td, column.meta?.class?.td]"
        >
          <slot :name="`${column.id || column.accessorKey}-cell`" :row="{ original }">{{
            value(original, column)
          }}</slot>
        </td>
      </tr>
      <tr v-if="!data.length">
        <td :colspan="columns.length" class="text-center text-muted" :class="ui?.td">
          データがありません
        </td>
      </tr>
    </tbody>
  </table>
</template>
