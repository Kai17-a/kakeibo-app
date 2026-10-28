import type { RouteLocationRaw } from "vue-router";
export interface NavigationMenuItem {
  label: string;
  to?: RouteLocationRaw;
  icon?: string;
  active?: boolean;
  exact?: boolean;
  onSelect?: () => void;
}
export interface TableColumn<T> {
  id?: string;
  accessorKey?: string;
  header?: string;
  cell?: (context: { row: { original: T } }) => unknown;
  meta?: { class?: { th?: string; td?: string } };
}
